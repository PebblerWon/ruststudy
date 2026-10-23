# Phase 1：数据层技术实现

> 本文档的所有接口规格均于 **2026-10-23 用 curl 实测验证**（贵州茅台 600519、上证指数等），实现时如遇字段不符，以实测为准并回写本文。

## 1.1 模块概述

### 目标

接入东方财富公开行情接口，提供批量快照（自选/指数）、全市场列表、历史 K 线、当日分时四类数据，配 TTL 缓存与轮询调度，为 UI 层提供唯一数据出口。

### 覆盖的 Rust 知识点

| 知识点 | 实践场景 |
|--------|---------|
| **reqwest** | 默认请求头注入、异步 GET、限速 |
| **serde / serde_json** | JSON 反序列化、`"-"` 容错、CSV 手工解析 |
| **tokio** | interval 轮询、令牌桶限速、后台任务 |
| **chrono** | 交易时段判断、K 线时间解析（Asia/Shanghai） |
| **thiserror** | 错误类型定义 |
| **纯函数设计** | secid 规则、单位换算、时段判断（易单测） |

依赖方向：`data → common`，`ui → data`。

---

## 1.2 项目初始化

```toml
# stock-data/Cargo.toml
[package]
name = "stock-data"
version = "0.1.0"
edition = "2024"

[dependencies]
tokio = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
wiremock = "0.6"        # 或 tokio-test + 手工 mock transport
tempfile = "3"
```

> 依赖版本统一放根 `Cargo.toml` 的 `[workspace.dependencies]`，子 crate `workspace = true` 继承（与 quant 一致）。

### 目录结构

```
stock-data/src/
├── lib.rs
├── common/
│   ├── mod.rs
│   ├── error.rs            # DataError
│   ├── models.rs           # Quote/IndexQuote/Kline/TimelinePoint/MarketSnapshot
│   └── trading_hours.rs    # TradingPhase
└── data/
    ├── mod.rs
    ├── eastmoney.rs        # EastmoneyClient
    ├── eastmoney_types.rs  # raw 响应类型
    ├── normalize.rs        # 单位归一
    ├── cache.rs            # TTL 缓存
    └── polling.rs          # 轮询服务
```

---

## 1.3 数据模型

```rust
// src/common/models.rs

/// 个股实时快照（出口已归一：市值亿元、成交额万元、成交量手）
pub struct Quote {
    pub code: String,          // 6 位代码
    pub name: String,
    pub market: u8,            // 0=深 1=沪
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,       // %
    pub volume: f64,           // 手
    pub amount: f64,           // 万元
    pub turnover_rate: f64,    // %
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub total_mv: f64,         // 亿元
    pub circ_mv: f64,          // 亿元
}

/// 指数快照
pub struct IndexQuote {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub amount: f64,           // 亿元
}

/// K 线（与 quant 的 Kline 一致，可直接拷贝 models.rs）
pub struct Kline {
    pub open_time: i64,        // Unix ms
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,           // 股（东财手 × 100）
    pub close_time: i64,
    pub quote_volume: f64,     // 元
    pub trades_count: u32,     // 东财无此字段，恒为 0
    pub is_closed: bool,
}

/// 分时点
pub struct TimelinePoint {
    pub time: String,          // "HH:MM"
    pub price: f64,
    pub avg_price: f64,
    pub volume: f64,           // 手
    pub amount: f64,           // 万元
}

/// 全市场统计（总览页）
pub struct MarketSnapshot {
    pub total: usize,
    pub up_count: usize,
    pub down_count: usize,
    pub flat_count: usize,
    pub limit_up: usize,       // 涨跌幅 ≥ 9.9%（近似）
    pub limit_down: usize,
    pub total_amount: f64,     // 亿元
}
```

**`"-"` 容错反序列化**（停牌/无数据时数值字段返回字符串 `"-"`）：

```rust
// src/data/eastmoney_types.rs
fn f64_or_zero<'de, D>(d: D) -> Result<f64, D::Error>
where D: serde::Deserializer<'de> {
    use serde_json::Value;
    Ok(match Value::deserialize(d)? {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.parse().unwrap_or(0.0),   // "-" → 0.0
        _ => 0.0,
    })
}
```

> 注意：`"-"` 归零与"真实价格为 0"不可区分。Quote 增加 `is_suspended: bool`（f2 为 `"-"` 时置 true），UI 显示"停牌"。

---

## 1.4 EastmoneyClient

### 1.4.1 通用要求（实测）

- **必须携带请求头**，否则返回 200 + 空 body：
  - `User-Agent`: 浏览器 UA
  - `Referer: https://quote.eastmoney.com/center/gridlist.html`
- 响应 UTF-8 JSON；数值字段 `fltt=2` 时为浮点，停牌时为字符串 `"-"`
- 非官方接口：令牌桶限速 ≥500ms/请求，失败退避重试

```rust
pub struct EastmoneyClient {
    http: reqwest::Client,   // DefaultHeaders: UA + Referer
    limiter: TokenBucket,    // ≥500ms 间隔
}

impl EastmoneyClient {
    /// 自选/个股批量快照（一次 ≤50 个 secid，超出分批）
    pub async fn quotes(&self, codes: &[&str]) -> Result<Vec<Quote>>;
    /// 8 大指数快照
    pub async fn index_quotes(&self) -> Result<Vec<IndexQuote>>;
    /// 全市场快照（clist 分页拉取 + 本地统计）
    pub async fn market_snapshot(&self) -> Result<MarketSnapshot>;
    /// 历史 K 线
    pub async fn klines(&self, code: &str, interval: Interval,
                        fqt: Adjust, beg: NaiveDate) -> Result<Vec<Kline>>;
    /// 当日分时
    pub async fn timeline(&self, code: &str, ndays: u8) -> Result<Timeline>;
}
```

### 1.4.2 secid 规则（纯函数，重点单测）

```rust
fn secid(code: &str) -> Result<String> {
    // 60xxxx / 68xxxx → "1.{code}"     沪
    // 00xxxx / 30xxxx → "0.{code}"     深
    // 指数由调用方显式传入完整 secid
    // 8xxxxx / 4xxxxx / 92xxxx → Err   北交所，不支持
}
```

### 1.4.3 令牌桶限速（简化实现）

```rust
struct TokenBucket { interval: Duration, last: tokio::sync::Mutex<Instant> }
impl TokenBucket {
    async fn acquire(&self) {
        let mut last = self.last.lock().await;
        let wait = self.interval.saturating_sub(last.elapsed());
        tokio::time::sleep(wait).await;
        *last = Instant::now();
    }
}
```

---

## 1.5 接口规格与字段映射（实测验证）

### 1.5.1 批量快照 ulist.np（自选 + 指数卡片）

```
GET https://push2.eastmoney.com/api/qt/ulist.np/get
    ?fltt=2&secids=1.000001,0.399001,0.399006,1.000688,1.000300,1.000016,1.000905,0.899050
    &fields=f2,f3,f4,f5,f6,f8,f12,f13,f14,f15,f16,f17,f18,f20,f21
```

字段：f2 最新价 / f3 涨跌幅% / f4 涨跌额 / f5 成交量(手) / f6 成交额(元) / f8 换手率% / f12 代码 / f13 市场 / f14 名称 / f15-f18 高低开昨收 / f20 总市值(元) / f21 流通市值(元)。

**常用指数 secid：** 上证指数 `1.000001`、深证成指 `0.399001`、创业板指 `0.399006`、科创50 `1.000688`、沪深300 `1.000300`、上证50 `1.000016`、中证500 `1.000905`、北证50 `0.899050`。

### 1.5.2 全市场列表 clist（总览统计 + 热力图 + 榜单）

```
GET https://push2.eastmoney.com/api/qt/clist/get
    ?pn={page}&pz=100&po=1&np=1&fltt=2&invt=2
    &fid=f3                                  # 排序字段：f3 涨跌幅 / f6 成交额 / f20 总市值
    &fs=m:1+t:2,m:0+t:6,m:0+t:80,m:1+t:23    # 沪深主板+创业板+科创板（不含北交）
    &fields=f2,f3,f5,f6,f8,f12,f13,f14,f20,f21
```

- 全市场约 5400 只，pz=100 分页约 55 次请求；响应 `data.total` 判断终止
- 涨跌家数：全量拉取后本地统计（60s TTL，与热力图共享快照）
- 涨跌停近似：f3 ≥ 9.9% 计涨停（主板 10%，创业板/科创 20% 需按 f13+f12 前缀区分，MVP 可先只统计主板口径并注明）

### 1.5.3 历史 K 线 kline

```
GET https://push2his.eastmoney.com/api/qt/stock/kline/get
    ?secid=1.600519
    &fields1=f1,f2,f3,f4,f5,f6
    &fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61
    &klt=101&fqt=1
    &beg=20240101&end=20500101
```

| 参数 | 说明 |
|------|------|
| `secid` | `1.` 沪 / `0.` 深 |
| `klt` | 1/5/15/30/60 分钟，101 日，102 周 |
| `fqt` | 0 不复权 / 1 前复权 / 2 后复权（指标与展示用前复权） |
| `beg`/`end` | YYYYMMDD |

**klines CSV 顺序（`fields2`）：** `f51 时间, f52 开, f53 收, f54 高, f55 低, f56 量(手), f57 额(元), f58 振幅, f59 涨跌幅, f60 涨跌额, f61 换手率`——**收在高位之前**，解析时勿按 开/高/收/低 习惯顺序取。

**三个实测坑：**

1. **`lmt` 在 `beg=0` 时未生效**（传 `lmt=2` 仍返回全量）。控制范围用 `beg`/`end`，"最近 N 根"客户端截取。
2. **前复权早期数据可能为负**（茅台 2001 年 `-312.68`）。解析不做正数校验；展示/指标前按需过滤。
3. **分钟线历史浅**：1 分钟仅当日（至多近 5 日）；5/15/30/60 分钟仅最近 ~32 个交易日（实测茅台 klt=5 共 1516 根 ≈ 1.5 个月），`beg` 无法突破。日线/周线上市至今全量（茅台 dktotal=6027）。

Kline 字段映射：`open_time` 由 f51 解析（日线补 09:30，Asia/Shanghai，Unix ms 存储）；`volume = f56 × 100`（手→股）；`close_time = open_time + 周期`（日线 15:00）；`quote_volume = f57`；`trades_count = 0`；`is_closed`：最后一根且未到收盘时间为 false。

### 1.5.4 分时 trends2

```
GET https://push2his.eastmoney.com/api/qt/stock/trends2/get
    ?secid=1.600519&fields1=f1,f2,f3,f7,f8
    &fields2=f51,f52,f53,f54,f55,f56,f57,f58&iscr=0&ndays=1
```

`trends` 每项 CSV：`时间, 开, 现价, 高, 低, 量(手), 额(元), 均价`；`data.preClose` 为昨收基准。实测样例：

```json
{"data":{"preClose":1318.49,"trends":["2026-10-23 09:31,1311.50,1321.00,1323.65,1310.00,986,129903055.00,1314.441"]}}
```

---

## 1.6 交易时段与轮询

```rust
// src/common/trading_hours.rs
pub enum TradingPhase { PreOpen, Auction, Morning, LunchBreak, Afternoon, Closed }
pub fn phase(now: NaiveTime, weekday: Weekday) -> TradingPhase {
    // 09:15-09:25 集合竞价 → Auction
    // 09:30-11:30 → Morning；11:30-13:00 → LunchBreak；13:00-15:00 → Afternoon
    // 周六日 → Closed（节假日表 MVP 省略）
}
```

边界单测：09:14/09:15/09:29/09:30/11:29/11:30/12:59/13:00/14:59/15:00/周六。

```rust
// src/data/polling.rs
pub struct PollingService { /* interval 按 TradingPhase 动态调整 */ }
// 盘中（Morning/Afternoon/Auction）: 5s；其余: 60s
// 网络失败：指数退避 1s→2s→4s→…→30s 封顶，成功后恢复
```

## 1.7 TTL 缓存

```rust
// src/data/cache.rs — 不引入外部依赖
pub struct TtlCache<V> { map: Mutex<HashMap<String, (Instant, Duration, V)>> }
impl<V: Clone> TtlCache<V> {
    pub async fn get_or_insert_with(&self, key: &str, ttl: Duration,
        f: impl AsyncFnOnce() -> Result<V>) -> Result<V>;
}
```

| 数据 | TTL |
|------|-----|
| 行情快照（quotes/index_quotes） | 5s |
| K 线 | 10min |
| 全市场快照 | 60s |

---

## 1.8 测试方案

| 对象 | 方式 |
|------|------|
| `secid()` | 沪 60/68、深 00/30、指数、拒绝 8/4/9 开头 |
| `normalize` | 元→万/亿、手→股；0/负值/极值 |
| CSV kline 解析 | fixture 含前复权负价（茅台 2001 年真实样例）；断言 OHLC 有序 high ≥ max(open, close) |
| `"-"` 容错 | 停牌 fixture → is_suspended=true，数值归零 |
| TradingPhase | 1.6 边界表 |
| HTTP 层 | wiremock + `tests/fixtures/eastmoney/{ulist,clist,kline,trends}.json` |
| 真实接口 | `#[ignore]` smoke test：茅台日线 OHLC 有序；指数快照 8 条 |
