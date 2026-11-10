# Phase 1：数据层技术实现

> 所有接口规格均于 **2026-10-23 用 curl 实测验证**（贵州茅台 600519、上证指数等；§1.8 北向资金为 **2026-11-10 补充实测**）。实现时如遇字段不符，以实测为准并回写本文。

## 1.1 模块概述

### 目标

接入东方财富公开行情接口，提供**批量快照（自选/指数）、全市场列表、历史 K 线、当日分时、北向资金（沪深港通）**五类数据，配 TTL 缓存与轮询调度，为 UI 层提供唯一数据出口。

### 模块划分与职责

| 文件                      | 职责                        | 关键类型/函数                                                                     | 对应任务                  |
| ------------------------- | --------------------------- | --------------------------------------------------------------------------------- | ------------------------- |
| `common/error.rs`         | 统一错误类型                | `DataError`、`Result<T>`                                                          | T1.2                      |
| `common/models.rs`        | 归一后的领域模型            | `Quote` `IndexQuote` `Kline` `TimelinePoint` `MarketSnapshot` `Interval` `Adjust` | T1.2                      |
| `common/trading_hours.rs` | 交易时段判断（纯函数）      | `TradingPhase`、`phase()`、`now_shanghai()`                                       | T3.1（定义在 Phase 1 建） |
| `data/eastmoney_types.rs` | 东财原始响应解析            | `take_data()`、`f64_field()` 等 helper、`parse_quote_row()`、kline/分时 CSV 解析  | T1.4                      |
| `data/normalize.rs`       | 单位归一（元→万/亿、手→股） | `yuan_to_wan()` 等                                                                | T1.4                      |
| `data/eastmoney.rs`       | HTTP 客户端                 | `EastmoneyClient`、`TokenBucket`、`secid()`                                       | T1.3/T1.4                 |
| `data/cache.rs`           | 内存 TTL 缓存               | `TtlCache<V>`                                                                     | T1.5                      |
| `data/polling.rs`         | 轮询调度（时段感知 + 退避） | `spawn_polling()`、`PollingHandle`                                                | T3.1                      |

### 依赖关系与数据流

```
common (error/models/trading_hours)     ← 零外部 IO，纯类型与纯函数
    ▲
data (eastmoney → eastmoney_types + normalize；cache；polling)
    ▲
stock-app (ui)
```

- `eastmoney_types.rs` 只做「JSON/CSV → 归一化模型」的**解析**，不发请求
- `eastmoney.rs` 只做「构造 URL → 限速 → 发请求 → 交给解析」的**编排**，不理解字段含义
- 单位归一**只在 client 出口做一次**，UI 层禁止再换算（单位契约：市值亿元、成交额万元、成交量手；Kline 例外——volume 为股，与 quant 的 Kline 模型一致）

---

## 1.2 项目初始化

### workspace 根 Cargo.toml

```toml
[workspace]
resolver = "2"
members = ["crates/stock-data", "crates/stock-app"]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "gzip"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"
tracing = "0.1"
```

### stock-data/Cargo.toml

```toml
[package]
name = "stock-data"
version = "0.1.0"
edition = "2024"

[lib]
name = "stock_data"
path = "src/lib.rs"

[dependencies]
tokio = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
wiremock = "0.6"
tempfile = "3"
```

### 目录结构与 lib.rs

```
stock-data/src/
├── lib.rs
├── common/
│   ├── mod.rs              # pub mod error; pub mod models; pub mod trading_hours;
│   ├── error.rs
│   ├── models.rs
│   └── trading_hours.rs
└── data/
    ├── mod.rs              # pub mod eastmoney; pub mod eastmoney_types;
    │                       # pub mod normalize; pub mod cache; pub mod polling;
    ├── eastmoney.rs
    ├── eastmoney_types.rs
    ├── normalize.rs
    ├── cache.rs
    └── polling.rs
```

```rust
// src/lib.rs
pub mod common;
pub mod data;

pub use common::{error::DataError, models::*};
```

---

## 1.3 错误类型与数据模型

### 1.3.1 error.rs — 统一错误

```rust
// src/common/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("网络请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),

    #[error("无效的股票代码: {0}")]
    InvalidCode(String),

    #[error("接口返回异常: {0}")]
    Api(String),
}

pub type Result<T> = std::result::Result<T, DataError>;
```

设计点：库层只定义这一种错误；分页中途失败、空响应等业务异常用 `DataError::Api(String)` 携带上下文（哪个接口、第几页）。

### 1.3.2 models.rs — 领域模型（全量代码）

```rust
// src/common/models.rs
use serde::{Deserialize, Serialize};

/// 个股实时快照（出口已归一：市值亿元、成交额万元、成交量手）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    pub code: String,          // 6 位代码，如 "600519"
    pub name: String,
    pub market: u8,            // 0=深 1=沪
    pub price: f64,
    pub change: f64,           // 涨跌额（元）
    pub change_pct: f64,       // 涨跌幅（%）
    pub volume: f64,           // 成交量（手）
    pub amount: f64,           // 成交额（万元）
    pub turnover_rate: f64,    // 换手率（%）
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub total_mv: f64,         // 总市值（亿元）
    pub circ_mv: f64,          // 流通市值（亿元）
    pub is_suspended: bool,    // 停牌（f2 为 "-" 时 true，数值字段归零）
}

/// 指数快照（总览页卡片）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexQuote {
    pub code: String,          // "000001"
    pub name: String,          // "上证指数"
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub amount: f64,           // 成交额（亿元）
}

/// K 线（字段与 quant 项目 Kline 一致，可直接拷贝）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Kline {
    pub open_time: i64,        // Unix ms（Asia/Shanghai 本地时间的毫秒时间戳）
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,           // 股（东财手 × 100）
    pub close_time: i64,       // Unix ms
    pub quote_volume: f64,     // 成交额（元）
    pub trades_count: u32,     // 东财无此字段，恒为 0
    pub is_closed: bool,
}

/// 分时点
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelinePoint {
    pub time: String,          // "HH:MM"
    pub price: f64,            // 现价
    pub avg_price: f64,        // 均价线
    pub volume: f64,           // 手
    pub amount: f64,           // 万元
}

/// 当日分时（含昨收基准，供涨跌幅着色）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Timeline {
    pub code: String,
    pub pre_close: f64,
    pub points: Vec<TimelinePoint>,
}

/// 全市场统计（总览页）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MarketSnapshot {
    pub total: usize,
    pub up_count: usize,
    pub down_count: usize,
    pub flat_count: usize,
    pub limit_up: usize,       // 近似口径，见 §1.5.2
    pub limit_down: usize,
    pub total_amount: f64,     // 亿元
}

/// K 线周期（东财无 4h，刻意不设 H4）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interval { M1, M5, M15, M30, H1, D1, W1 }

impl Interval {
    /// 东财 klt 参数
    pub fn klt(self) -> u32 {
        match self {
            Interval::M1 => 1,
            Interval::M5 => 5,
            Interval::M15 => 15,
            Interval::M30 => 30,
            Interval::H1 => 60,
            Interval::D1 => 101,
            Interval::W1 => 102,
        }
    }

    /// 分钟粒度时长；日线/周线返回 0（close_time 特殊处理）
    pub fn minutes(self) -> i64 {
        match self {
            Interval::M1 => 1,
            Interval::M5 => 5,
            Interval::M15 => 15,
            Interval::M30 => 30,
            Interval::H1 => 60,
            _ => 0,
        }
    }
}

/// 复权方式（东财 fqt 参数）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adjust {
    None = 0,      // 不复权
    Forward = 1,   // 前复权（指标与展示默认用这个）
    Backward = 2,  // 后复权
}

impl Adjust {
    pub fn fqt(self) -> u32 { self as u32 }
}
```

设计点：

- **`Interval` 刻意没有 H4**——东财无 4h K 线（见 §1.5.3 坑 3），类型层面杜绝
- `Quote::is_suspended`：`"-"` 归零后与"真实价格为 0"不可区分，必须单独打标（UI 显示"停牌"）
- `Kline` 保持与 quant 的 `Kline` 字段一致：将来迁移指标库（T4.2）或 KlineStore 时零适配成本

---

## 1.4 客户端骨架：请求头、限速、secid

> 对应 T1.3。本节只搭骨架（构造 + 限速 + secid + HTTP helper），五个数据方法的实现放 §1.5.5。

### 1.4.1 EastmoneyClient 结构与构造

```rust
// src/data/eastmoney.rs
use crate::common::error::{DataError, Result};
use crate::common::models::*;
use crate::data::eastmoney_types as raw;
use crate::data::normalize as norm;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::Mutex;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                  (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
const REFERER: &str = "https://quote.eastmoney.com/center/gridlist.html";

#[derive(Clone)]
pub struct EastmoneyClient {
    http: reqwest::Client,
    limiter: TokenBucket,
    base_q: String,     // push2 域名（实时快照类）
    base_his: String,   // push2his 域名（K 线/分时类）
}

impl EastmoneyClient {
    /// 生产构造
    pub fn new() -> Result<Self> {
        Self::with_base_url("https://push2.eastmoney.com/api/qt".into(),
                            "https://push2his.eastmoney.com/api/qt".into())
    }

    /// 测试构造：wiremock 注入 base_url（见 §1.7.3）
    pub fn with_base_url(base_q: String, base_his: String) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(UA)
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert("Referer", REFERER.parse().unwrap());
                h
            })
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self { http, limiter: TokenBucket::new(Duration::from_millis(500)), base_q, base_his })
    }

    /// 统一入口：限速 → GET → 解析 JSON（空 body 视为接口异常）
    async fn get_json(&self, url: &str) -> Result<Value> {
        self.limiter.acquire().await;
        let text = self.http.get(url).send().await?.error_for_status()?.text().await?;
        if text.trim().is_empty() {
            // 实测：缺 Referer/被风控时返回 200 + 空 body
            return Err(DataError::Api(format!("接口返回空响应: {url}")));
        }
        Ok(serde_json::from_str(&text)?)
    }
}
```

> **实测结论**：`push2.eastmoney.com` 的 clist 缺 `Referer` 时返回空 body（不是报错）；`push2his` 的 kline/trends2 对头不敏感，但统一加头无害。请求失败时优先怀疑请求头。

### 1.4.2 TokenBucket 限速

```rust
// src/data/eastmoney.rs（续）
#[derive(Clone)]
pub struct TokenBucket {
    interval: Duration,
    last: Arc<Mutex<Instant>>,
}

impl TokenBucket {
    pub fn new(interval: Duration) -> Self {
        Self { interval, last: Arc::new(Mutex::new(Instant::now() - interval)) }
    }

    /// 保证两次调用间隔 ≥ interval（严格串行限速，而非并发桶）
    pub async fn acquire(&self) {
        let mut last = self.last.lock().await;
        let wait = self.interval.saturating_sub(last.elapsed());
        tokio::time::sleep(wait).await;
        *last = Instant::now();
    }
}
```

设计点：用 `tokio::sync::Mutex` 持锁跨 sleep——这**正是想要的语义**（并发请求在锁上排队，每个间隔最多放行一个），与 rustkv 里学的"锁内不做长等待"不冲突，因为这里等的就是限速本身。

### 1.4.3 secid 市场规则（纯函数）

```rust
// src/data/eastmoney.rs（续）

/// 沪深主板 + 创业板 + 科创板 → secid；北交所/非法输入 → Err
pub fn secid(code: &str) -> Result<String> {
    let code = code.trim();
    if code.len() != 6 || !code.bytes().all(|b| b.is_ascii_digit()) {
        return Err(DataError::InvalidCode(code.to_string()));
    }
    match &code[..2] {
        "60" | "68" => Ok(format!("1.{code}")),   // 沪主板 / 科创
        "00" | "30" => Ok(format!("0.{code}")),   // 深主板 / 创业
        "83" | "87" | "88" | "43" | "92" => {
            Err(DataError::InvalidCode(format!("{code}（北交所暂不支持）")))
        }
        _ => Err(DataError::InvalidCode(code.to_string())),
    }
}

/// 8 大指数 secid 常量（总览页顺序即展示顺序）
pub const INDEX_SECIDS: [&str; 8] = [
    "1.000001", // 上证指数
    "0.399001", // 深证成指
    "0.399006", // 创业板指
    "1.000688", // 科创50
    "1.000300", // 沪深300
    "1.000016", // 上证50
    "1.000905", // 中证500
    "0.899050", // 北证50（北交所指数本身可用，只是北交个股不支持）
];
```

> 指数与个股不同：指数走显式 secid 常量（`1.000001` 等），不经过 `secid()` 规则函数。

---

## 1.5 数据解析与归一

> 对应 T1.4。先建原始层解析（1.5.1-1.5.4），再实现五个数据方法（1.5.5）。

### 1.5.1 原始层设计：为什么不定义强类型 struct

东财响应字段是 `f2`/`f3` 这种动态键，且同一字段**类型不稳定**（数值 `3.03`、停牌时字符串 `"-"`、偶尔 `null`）。如果用 `#[derive(Deserialize)]` 的 struct + 自定义 deserializer，要给 15 个字段各写一遍容错。**改用 `serde_json::Value` + 提取 helper**，容错逻辑只写一次：

```rust
// src/data/eastmoney_types.rs
use crate::common::error::{DataError, Result};
use crate::common::models::*;

/// 通用响应壳：{"rc":0,"data":{...}}，data 可能为 null
pub fn take_data(v: &Value, api: &str) -> Result<&Value> {
    v.get("data")
        .filter(|d| !d.is_null())
        .ok_or_else(|| DataError::Api(format!("{api}: data 为空")))
}

/// 提取数值字段：Number 直接取；String 尝试 parse（"-" 等失败归 0）；null 归 0
pub fn f64_field(v: &Value, key: &str) -> f64 {
    match v.get(key) {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// 是否为 "-"（停牌占位），用于打 is_suspended 标
pub fn is_dash(v: &Value, key: &str) -> bool {
    matches!(v.get(key), Some(Value::String(s)) if s == "-")
}

pub fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

pub fn int_field(v: &Value, key: &str) -> i64 {
    f64_field(v, key) as i64
}
```

> 注意 `f64_field` 对 `"-"` 返回 0.0 且**丢失了"是否停牌"信息**——所以 `Quote` 才需要 `is_suspended` 字段，由 `is_dash(f2)` 单独判定。这两个函数必须配套使用。

### 1.5.2 ulist / clist 行解析

两个接口的行结构相同（`diff` 数组，元素是 f 字段字典），共用一个解析函数：

```rust
// src/data/eastmoney_types.rs（续）
use crate::data::normalize as norm;

/// clist/ulist 行 → Quote（出口单位已归一）
pub fn parse_quote_row(row: &Value) -> Quote {
    let suspended = is_dash(row, "f2");
    Quote {
        code: str_field(row, "f12").to_string(),
        name: str_field(row, "f14").to_string(),
        market: int_field(row, "f13") as u8,
        price: f64_field(row, "f2"),
        change: f64_field(row, "f4"),
        change_pct: f64_field(row, "f3"),
        volume: f64_field(row, "f5"),                    // 手
        amount: norm::yuan_to_wan(f64_field(row, "f6")), // 元 → 万元
        turnover_rate: f64_field(row, "f8"),
        open: f64_field(row, "f17"),
        high: f64_field(row, "f15"),
        low: f64_field(row, "f16"),
        pre_close: f64_field(row, "f18"),
        total_mv: norm::yuan_to_yi(f64_field(row, "f20")),
        circ_mv: norm::yuan_to_yi(f64_field(row, "f21")),
        is_suspended: suspended,
    }
}
```

实测响应样例（可直接做 fixture）：

```json
{
  "rc": 0,
  "data": {
    "total": 3488,
    "diff": [
      {
        "f2": 3.03,
        "f3": 10.18,
        "f5": 505496,
        "f6": 152967019.1,
        "f8": 6.77,
        "f12": "002535",
        "f13": 0,
        "f14": "林州重机",
        "f15": 3.03,
        "f16": 3.0,
        "f17": 3.01,
        "f18": 2.75,
        "f20": 2429099714,
        "f21": 2261402731
      }
    ]
  }
}
```

### 1.5.3 kline CSV 解析

```rust
// src/data/eastmoney_types.rs（续）
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, TimeZone};

const SHANGHAI: FixedOffset = FixedOffset::east_opt(8 * 3600).unwrap();

/// 解析一行 kline CSV：
/// "2024-01-02,1685.00,1685.90,1698.00,1670.11,30726,5172858000.00,1.72,-0.33,-5.62,0.24"
/// 顺序（fields2）：f51 时间, f52 开, f53 收, f54 高, f55 低, f56 量(手), f57 额(元),
///                 f58 振幅, f59 涨跌幅, f60 涨跌额, f61 换手率
/// ⚠️ 收盘价在开盘价之后、最高价之前——不是 开/高/收/低！
pub fn parse_kline_line(line: &str, interval: Interval) -> Result<Kline> {
    let p: Vec<&str> = line.split(',').collect();
    if p.len() < 7 {
        return Err(DataError::Api(format!("kline 字段数不足: {line}")));
    }

    // 时间解析：日线 "YYYY-MM-DD"（补 09:30 开盘），分钟线 "YYYY-MM-DD HH:MM"
    // 两个分支都显式产出 (NaiveDateTime, NaiveDateTime)，避免 if/else 类型不一致
    let (open_dt, close_dt): (NaiveDateTime, NaiveDateTime) = if interval.minutes() > 0 {
        let t = NaiveDateTime::parse_from_str(p[0], "%Y-%m-%d %H:%M")
            .map_err(|e| DataError::Api(format!("时间解析失败 {line}: {e}")))?;
        (t, t + chrono::Duration::minutes(interval.minutes()))
    } else {
        let d = NaiveDate::parse_from_str(p[0], "%Y-%m-%d")
            .map_err(|e| DataError::Api(format!("日期解析失败 {line}: {e}")))?;
        // 日/周线：open 取当日 09:30，close 取 15:00
        let open = d
            .and_hms_opt(9, 30, 0)
            .ok_or_else(|| DataError::Api(format!("无效日期 {line}")))?;
        let close = d
            .and_hms_opt(15, 0, 0)
            .ok_or_else(|| DataError::Api(format!("无效日期 {line}")))?;
        (open, close)
    };

    let open: f64 = p[1].parse().unwrap_or(0.0);
    let close: f64 = p[2].parse().unwrap_or(0.0);
    let high: f64 = p[3].parse().unwrap_or(0.0);
    let low: f64 = p[4].parse().unwrap_or(0.0);

    Ok(Kline {
        open_time: SHANGHAI.from_local_datetime(&open_dt).unwrap().timestamp_millis(),
        open,
        high,
        low,
        close,
        volume: p[5].parse::<f64>().unwrap_or(0.0) * 100.0,  // 手 → 股
        close_time: SHANGHAI.from_local_datetime(&close_dt).unwrap().timestamp_millis(),
        quote_volume: p[6].parse().unwrap_or(0.0),           // 元
        trades_count: 0,                                     // 东财无此字段
        is_closed: true,                                     // 由 client 对最后一根修正
    })
}

/// 最后一根未收盘的 K 线打上 is_closed=false
/// （盘中拉取时，最后一根的时间窗口还没走完）
pub fn mark_last_bar_open(klines: &mut [Kline], now_ms: i64) {
    if let Some(last) = klines.last_mut() {
        if now_ms < last.close_time {
            last.is_closed = false;
        }
    }
}
```

实测响应样例（前复权，注意 2001 年早期数据为负——解析不做正数校验）：

```json
{
  "data": {
    "code": "600519",
    "market": 1,
    "name": "贵州茅台",
    "dktotal": 6027,
    "klines": [
      "2024-01-02,1685.00,1685.90,1698.00,1670.11,30726,5172858000.00,1.72,-0.33,-5.62,0.24",
      "2001-08-27,-312.68,-312.47,-312.03,-313.01,406318,1410347000.00,-0.31,0.26,0.83,56.83"
    ]
  }
}
```

### 1.5.4 trends2 分时解析

```rust
// src/data/eastmoney_types.rs（续）
/// 一行分时 CSV："2026-10-23 09:31,1311.50,1321.00,1323.65,1310.00,986,129903055.00,1314.441"
/// 顺序（fields2）：f51 时间, f52 开, f53 现价, f54 高, f55 低, f56 量(手), f57 额(元), f58 均价
pub fn parse_timeline_line(line: &str) -> Option<TimelinePoint> {
    let p: Vec<&str> = line.split(',').collect();
    if p.len() < 8 { return None; }
    Some(TimelinePoint {
        time: p[0].split(' ').nth(1)?.to_string(),  // "09:31"
        price: p[2].parse().ok()?,
        avg_price: p[7].parse().ok()?,
        volume: p[5].parse().unwrap_or(0.0),                       // 手
        amount: norm::yuan_to_wan(p[6].parse().unwrap_or(0.0)),    // 元 → 万元
    })
}
```

实测响应样例：

```json
{
  "data": {
    "code": "600519",
    "preClose": 1318.49,
    "trends": [
      "2026-10-23 09:30,1311.84,1311.84,1311.84,1311.84,1049,137612016.00,1311.840",
      "2026-10-23 09:31,1311.50,1321.00,1323.65,1310.00,986,129903055.00,1314.441"
    ]
  }
}
```

### 1.5.5 normalize.rs — 单位归一

```rust
// src/data/normalize.rs

/// 元 → 万元
pub fn yuan_to_wan(yuan: f64) -> f64 { yuan / 1e4 }

/// 元 → 亿元
pub fn yuan_to_yi(yuan: f64) -> f64 { yuan / 1e8 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_conversion() {
        assert_eq!(yuan_to_wan(1.529_670_191e8), 15_296.701_91);
        assert_eq!(yuan_to_yi(2.429_099_714e9), 24.290_997_14);
        assert_eq!(yuan_to_wan(0.0), 0.0);
        assert_eq!(yuan_to_yi(-1.0e8), -1.0);   // 市值不会为负，但函数不设防
    }
}
```

### 1.5.6 EastmoneyClient 五个数据方法（完整实现）

```rust
// src/data/eastmoney.rs（续）
impl EastmoneyClient {
    /// 自选/个股批量快照（ulist.np，一次 ≤50 个 secid，自动分批）
    pub async fn quotes(&self, codes: &[&str]) -> Result<Vec<Quote>> {
        let secids: Vec<String> = codes.iter().map(|c| secid(c)).collect::<Result<_>>()?;
        let mut out = Vec::with_capacity(secids.len());
        for batch in secids.chunks(50) {
            let url = format!(
                "{}/ulist.np/get?fltt=2&invt=2&secids={}&fields=f2,f3,f4,f5,f6,f8,f12,f13,f14,f15,f16,f17,f18,f20,f21",
                self.base_q,
                batch.join(","),
            );
            let v = self.get_json(&url).await?;
            let data = raw::take_data(&v, "ulist")?;
            out.extend(data["diff"].as_array().unwrap_or(&vec![]).iter().map(raw::parse_quote_row));
        }
        Ok(out)
    }

    /// 8 大指数快照（总览页）
    pub async fn index_quotes(&self) -> Result<Vec<IndexQuote>> {
        let url = format!(
            "{}/ulist.np/get?fltt=2&secids={}&fields=f2,f3,f4,f6,f12,f14",
            self.base_q,
            INDEX_SECIDS.join(","),
        );
        let v = self.get_json(&url).await?;
        let data = raw::take_data(&v, "ulist-index")?;
        let rows = data["diff"].as_array().unwrap_or(&vec![]);
        Ok(rows.iter().map(|r| IndexQuote {
            code: raw::str_field(r, "f12").to_string(),
            name: raw::str_field(r, "f14").to_string(),
            price: raw::f64_field(r, "f2"),
            change: raw::f64_field(r, "f4"),
            change_pct: raw::f64_field(r, "f3"),
            amount: norm::yuan_to_yi(raw::f64_field(r, "f6")),
        }).collect())
    }

    /// 全市场快照（clist 分页拉全量 + 本地统计）
    /// 沪主板+深主板+创业板+科创板+北交所，在市约 5580 只
    /// （官方中上协 2026-09 口径 5579 家上市公司；我方为证券口径）
    /// ⚠️ 1. fs 不能用裸 m:0+t:81 —— 会混入新三板 6700+ 僵尸品种；
    ///       北交所需用 m:0+t:81+s:2048 精确圈定
    /// ⚠️ 2. 市场过滤符会附带历史退市股档案（约 350 条，简称带"退"/"PT"，f2 返 "-"），
    ///       已按 is_suspended 剔除，不参与 total 与涨跌统计
    /// ⚠️ 3. B 股（沪 m:1+t:3 / 深 m:0+t:7，共 114 条）不在统计内：B 股流动性趋零，涨跌无参考价值
    pub async fn market_snapshot(&self) -> Result<MarketSnapshot> {
        let fs = "m:1+t:2,m:0+t:6,m:0+t:80,m:1+t:23,m:0+t:81+s:2048";
        let fields = "f2,f3,f5,f6,f8,f12,f13,f14,f20,f21";
        let mut total = 0usize;
        let mut rows: Vec<Quote> = Vec::new();
        for pn in 1..=100usize {
            let url = format!(
                "{}/clist/get?pn={pn}&pz=100&po=1&np=1&fltt=2&invt=2&fid=f3&fs={fs}&fields={fields}",
                self.base_q,
            );
            let v = self.get_json(&url).await?;
            let data = raw::take_data(&v, "clist")?;
            total = data["total"].as_u64().unwrap_or(0) as usize;
            let page: Vec<Quote> = data["diff"].as_array().unwrap_or(&vec![])
                .iter().map(raw::parse_quote_row).collect();
            let page_len = page.len();
            rows.extend(page);
            if page_len == 0 || rows.len() >= total { break; }
        }
        tracing::debug!(total, fetched = rows.len(), "market_snapshot done");
        if rows.len() < total {
            tracing::warn!(fetched = rows.len(), total,
                "market_snapshot 分页未拉全，涨跌统计可能失真");
        }

        // 停牌股与退市股档案的 f2 均返回 "-"（is_suspended），
        // 二者都不该计入在市家数与涨跌统计，统一剔除
        let active: Vec<&Quote> = rows.iter().filter(|q| !q.is_suspended).collect();

        Ok(MarketSnapshot {
            // 在市证券数（含真平盘）；停牌与退市档案不计入，保证 up+down+flat == total
            total: active.len(),
            up_count: active.iter().filter(|q| q.change_pct > 0.0).count(),
            down_count: active.iter().filter(|q| q.change_pct < 0.0).count(),
            flat_count: active.iter().filter(|q| q.change_pct == 0.0).count(),
            // 近似口径：统一按 ≥9.9% 计（创业板/科创 20cm 会被高估，MVP 接受，UI 注明）
            limit_up: active.iter().filter(|q| q.change_pct >= 9.9).count(),
            limit_down: active.iter().filter(|q| q.change_pct <= -9.9).count(),
            total_amount: rows.iter().map(|q| q.amount).sum::<f64>() / 1e4, // 万元→亿元
        })
    }

    /// 历史 K 线（默认前复权）
    /// ⚠️ 实测 `lmt` 在 beg=0 时未生效，控制范围只靠 beg/end；"最近 N 根"由调用方截取
    pub async fn klines(&self, code: &str, interval: Interval,
                        adjust: Adjust, beg: chrono::NaiveDate) -> Result<Vec<Kline>> {
        let id = secid(code)?;
        let url = format!(
            "{}/stock/kline/get?secid={id}&fields1=f1,f2,f3,f4,f5,f6\
             &fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61\
             &klt={klt}&fqt={fqt}&beg={beg}&end=20500101",
            self.base_his,
            klt = interval.klt(),
            fqt = adjust.fqt(),
            beg = beg.format("%Y%m%d"),
        );
        let v = self.get_json(&url).await?;
        let data = raw::take_data(&v, "kline")?;
        let mut out: Vec<Kline> = data["klines"].as_array().unwrap_or(&vec![])
            .iter()
            .filter_map(|l| l.as_str())
            .map(|l| raw::parse_kline_line(l, interval))
            .collect::<Result<_>>()?;
        raw::mark_last_bar_open(&mut out, crate::common::trading_hours::now_ms_shanghai());
        Ok(out)
    }

    /// 当日分时（ndays ≤ 5，MVP 固定 1）
    pub async fn timeline(&self, code: &str, ndays: u8) -> Result<Timeline> {
        let id = secid(code)?;
        let url = format!(
            "{}/stock/trends2/get?secid={id}&fields1=f1,f2,f3,f7,f8\
             &fields2=f51,f52,f53,f54,f55,f56,f57,f58&iscr=0&ndays={ndays}",
            self.base_his,
        );
        let v = self.get_json(&url).await?;
        let data = raw::take_data(&v, "trends2")?;
        Ok(Timeline {
            code: raw::str_field(data, "code").to_string(),
            pre_close: raw::f64_field(data, "preClose"),
            points: data["trends"].as_array().unwrap_or(&vec![])
                .iter()
                .filter_map(|l| l.as_str())
                .filter_map(raw::parse_timeline_line)
                .collect(),
        })
    }
}
```

> `market_snapshot` 的 `total_amount`：`q.amount` 已是万元，在市约 5600 只，除以 1e4 转回亿元，注意别二次换算。
>
> ⚠️ 已踩过的坑（2026-11 修复）：
>
> 1. fs 误带裸 `m:0+t:81`（北交所+新三板），拉进 6700+ 僵尸品种，`f3="-"` 归 0 后全被计入平盘；
> 2. total 1.2 万 > 100 页×pz=100=1 万，按 `fid=f3` 降序截断，下跌品种全排在未拉取的尾部 → down_count=0；
> 3. UI 显示层 `format!("{:.2}亿", x.to_string())`，`{:.2}` 对字符串是截前 2 字符而非保留小数，1 万亿显示成 "10亿"。
> 4. 与官方口径对账（中上协 2026-09 月报：5579 家 = 沪 2323 + 深 2908 + 北 348）：
>    - 官方是**公司口径**（A+B 双重上市计 1 家，含纯 B 7 家），东财 clist 是**证券口径**（A、B 各一条）；
>    - 东财市场过滤符会附带**历史退市股档案**（乐视退、金亚退等约 350 条，f2 返 "-"，已被 is_suspended 剔除）；
>    - 北交所需用 `m:0+t:81+s:2048`（365 只），裸 `m:0+t:81` 会混入新三板 6700+ 僵尸；
>    - 对账方法：深交所官网 `www.szse.cn/api/report/ShowReport/data?SHOWTYPE=JSON&CATALOGID=1110&TABKEY=tab1`（在市 A 股 2906 条）与东财列表做代码差集，多出 189 条全部为退市档案、缺失 0 条，账目闭环。

---

## 1.6 缓存、交易时段与轮询

> 对应 T1.5 与 T3.1。trading_hours 在 Phase 1 建文件（纯函数 + 单测），polling 的 UI 集成在 Phase 3。

### 1.6.1 trading_hours.rs — 交易时段（纯函数）

```rust
// src/common/trading_hours.rs
use chrono::{Datelike, FixedOffset, NaiveTime, TimeZone, Weekday};

const SHANGHAI: FixedOffset = FixedOffset::east_opt(8 * 3600).unwrap();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingPhase {
    PreOpen,     // 开盘前（含 09:25-09:30 连续竞价前的静默段）
    Auction,     // 09:15-09:25 集合竞价
    Morning,     // 09:30-11:30
    LunchBreak,  // 11:30-13:00
    Afternoon,   // 13:00-15:00
    Closed,      // 收盘 / 周末
}

/// 当前上海时间（时间 + 星期）
pub fn now_shanghai() -> (NaiveTime, Weekday) {
    let t = chrono::Utc::now().with_timezone(&SHANGHAI);
    (t.time(), t.weekday())
}

/// 当前 Unix ms（供 mark_last_bar_open 用）
pub fn now_ms_shanghai() -> i64 {
    chrono::Utc::now().with_timezone(&SHANGHAI).timestamp_millis()
}

pub fn phase(now: NaiveTime, weekday: Weekday) -> TradingPhase {
    use TradingPhase::*;
    if matches!(weekday, Weekday::Sat | Weekday::Sun) {
        return Closed;
    }
    let (h, m, _) = (now.hour(), now.minute(), now.second());
    let mins = h * 60 + m;
    match mins {
        0..=554    => PreOpen,     // < 09:15
        555..=565  => Auction,     // 09:15-09:25
        566..=569  => PreOpen,     // 09:25-09:30
        570..=690  => Morning,     // 09:30-11:30
        691..=779  => LunchBreak,  // 11:30-13:00
        780..=900  => Afternoon,   // 13:00-15:00
        _          => Closed,
    }
}

impl TradingPhase {
    /// 该时段的轮询间隔（秒）
    pub fn poll_interval_secs(self) -> u64 {
        match self {
            TradingPhase::Morning | TradingPhase::Afternoon | TradingPhase::Auction => 5,
            _ => 60,
        }
    }
}
```

> 用"分钟数区间"而不是多次比较 `NaiveTime`，边界一目了然：555=09:15、570=09:30、690=11:30、780=13:00、900=15:00。节假日表 MVP 省略（周末以外会白轮询，仅浪费流量不报错）。

### 1.6.2 cache.rs — TTL 缓存

```rust
// src/data/cache.rs
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct Entry<V> {
    at: Instant,
    ttl: Duration,
    value: V,
}

/// 进程内 TTL 缓存。MVP 无单飞（single-flight）：
/// 同 key 过期瞬间并发请求会重复打接口，被令牌桶限速兜住，可接受。
pub struct TtlCache<V> {
    entries: Mutex<HashMap<String, Entry<V>>>,
}

impl<V: Clone> TtlCache<V> {
    pub fn new() -> Self {
        Self { entries: Mutex::new(HashMap::new()) }
    }

    pub async fn get(&self, key: &str) -> Option<V> {
        let mut map = self.entries.lock().await;
        match map.get(key) {
            Some(e) if e.at.elapsed() < e.ttl => Some(e.value.clone()),
            Some(_) => { map.remove(key); None }   // 过期即清除
            None => None,
        }
    }

    pub async fn insert(&self, key: &str, value: V, ttl: Duration) {
        self.entries.lock().await.insert(
            key.to_string(),
            Entry { at: Instant::now(), ttl, value },
        );
    }

    /// 命中直接返回；未命中执行 f 并缓存
    pub async fn get_or_insert_with<F, Fut, E>(
        &self, key: &str, ttl: Duration, f: F,
    ) -> std::result::Result<V, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<V, E>>,
    {
        if let Some(v) = self.get(key).await {
            return Ok(v);
        }
        let v = f().await?;
        self.insert(key, v.clone(), ttl).await;
        Ok(v)
    }
}
```

TTL 分级（client 持有一个 `TtlCache<Value>` 存原始响应即可，单位换算幂等）：

| key 前缀                            | TTL   |
| ----------------------------------- | ----- |
| `ulist:{secids}`                    | 5s    |
| `kline:{secid}:{klt}:{fqt}:{beg}`   | 10min |
| `clist:snapshot`                    | 60s   |
| `NB_SUMMARY`（北向当日概览）        | 5s    |
| `NB_HIST:{channel}`（北向历史日线） | 30min |

### 1.6.3 polling.rs — 轮询服务（Phase 1 建骨架，Phase 3 接 UI）

```rust
// src/data/polling.rs
use crate::common::error::DataError;
use crate::common::trading_hours::{self, TradingPhase};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// 轮询句柄：drop 不停止任务，需调用 stop()（watch 通知）
pub struct PollingHandle {
    stop_tx: watch::Sender<bool>,
}

impl PollingHandle {
    pub fn stop(&self) { let _ = self.stop_tx.send(true); }
}

/// 启动轮询任务（通用框架：轮询什么由 fetch 决定）。
/// - `fetch`：每个 tick 调用一次，产出本次要执行的异步取数调用（如 client.quotes / klines）
/// - `on_data`：成功后在 tokio 线程被调用，Phase 3 中内部做 tx.try_send → gpui 唤醒
/// 失败退避：1s→2s→4s→…→30s 封顶，成功后恢复时段标准间隔。
pub fn spawn_polling<T, Fut, F, G>(fetch: G, on_data: F) -> PollingHandle
where
    G: Fn() -> Fut + Send + 'static,                       // 每 tick 调一次，产出新 future
    Fut: Future<Output = Result<T, DataError>> + Send + 'static,
    F: Fn(T) + Send + 'static,                             // 成功后怎么消费
    T: Send + 'static,
{
    let (stop_tx, stop_rx) = watch::channel(false);
    tokio::spawn(async move {
        let mut backoff_secs = 0u64;
        loop {
            if *stop_rx.borrow() { return; }
            let phase = { let (t, w) = trading_hours::now_shanghai(); trading_hours::phase(t, w) };
            let period = if backoff_secs > 0 {
                backoff_secs
            } else if phase == TradingPhase::Morning || phase == TradingPhase::Afternoon {
                5
            } else {
                60
            };
            tokio::time::sleep(Duration::from_secs(period)).await;

            match fetch().await {
                Ok(data) => { backoff_secs = 0; on_data(data); }
                Err(e) => {
                    backoff_secs = (backoff_secs * 2).max(1).min(30);
                    tracing::warn!(error = %e, backoff_secs, "polling failed");
                }
            }
        }
    });
    PollingHandle { stop_tx }
}
```

设计点：

- **泛化模式 `Fn() -> Fut`（2026-10-30 定稿）**：稳定版 Rust 没有 async 闭包，用「返回 future 的闭包」表达"每个 tick 要执行的异步调用"。三个约束：fetch 必须是 `Fn`（循环多次调用，不能 move 捕获物，只能 clone——`Arc::clone` 纳秒级）；返回的 Fut 必须 `'static`（`tokio::spawn` 要求），async move 块内自持 Arc 即可满足；错误统一走 `Result<_, DataError>` 进退避分支。行情场景的封装（start_polling_quotes）见 [03_integration.md §3.3.2](03_integration.md)，换数据源 = 换一个 fetch 闭包
- **为什么不用 `interval` 而用 `sleep`**：时段切换（盘中 5s ↔ 收盘 60s）和退避都需要动态改周期，`tokio::interval` 改周期要用 `MissedTickBehavior` 绕，直接 sleep 更直白
- **退避放在 sleep 侧而不是重试侧**：失败后下一轮直接等更久，天然满足"失败→降频"
- 关闭用 `watch` channel 而非引入 `tokio_util::CancellationToken` 新依赖；Phase 3 集成 gpui 时若 quant 已有 CancellationToken 经验可替换

---

## 1.7 测试方案

### 1.7.1 纯函数单测（`#[cfg(test)]` 模块内）

```rust
// src/data/eastmoney.rs 底部
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::error::DataError;

    #[test]
    fn secid_rules() {
        assert_eq!(secid("600519").unwrap(), "1.600519");  // 沪主板
        assert_eq!(secid("688981").unwrap(), "1.688981");  // 科创
        assert_eq!(secid("000001").unwrap(), "0.000001");  // 深主板
        assert_eq!(secid("300750").unwrap(), "0.300750");  // 创业
        assert!(secid("832566").is_err());                 // 北交 → 拒绝
        assert!(matches!(secid("832566"), Err(DataError::InvalidCode(_))));
        assert!(secid("60051").is_err());                  // 长度错
        assert!(secid("60051A").is_err());                 // 非数字
    }
}

// src/common/trading_hours.rs 底部
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn t(h: u32, m: u32) -> NaiveTime { NaiveTime::from_hms_opt(h, m, 0).unwrap() }

    #[test]
    fn phase_boundaries() {
        let wed = Weekday::Wed;
        assert_eq!(phase(t(9, 14), wed), TradingPhase::PreOpen);
        assert_eq!(phase(t(9, 15), wed), TradingPhase::Auction);
        assert_eq!(phase(t(9, 25), wed), TradingPhase::Auction);
        assert_eq!(phase(t(9, 26), wed), TradingPhase::PreOpen);
        assert_eq!(phase(t(9, 30), wed), TradingPhase::Morning);
        assert_eq!(phase(t(11, 30), wed), TradingPhase::Morning);   // 11:30 仍属早盘末
        assert_eq!(phase(t(11, 31), wed), TradingPhase::LunchBreak);
        assert_eq!(phase(t(13, 0), wed), TradingPhase::Afternoon);
        assert_eq!(phase(t(15, 0), wed), TradingPhase::Afternoon);
        assert_eq!(phase(t(15, 1), wed), TradingPhase::Closed);
        assert_eq!(phase(t(10, 0), Weekday::Sat), TradingPhase::Closed);
    }

    #[test]
    fn poll_interval() {
        assert_eq!(TradingPhase::Morning.poll_interval_secs(), 5);
        assert_eq!(TradingPhase::Closed.poll_interval_secs(), 60);
    }
}

// kline CSV 解析（含前复权负价）放 eastmoney_types.rs 底部
#[cfg(test)]
mod kline_tests {
    use super::*;

    #[test]
    fn parse_daily_bar() {
        let k = parse_kline_line(
            "2024-01-02,1685.00,1685.90,1698.00,1670.11,30726,5172858000.00,1.72,-0.33,-5.62,0.24",
            Interval::D1,
        ).unwrap();
        assert_eq!((k.open, k.close, k.high, k.low), (1685.00, 1685.90, 1698.00, 1670.11));
        assert_eq!(k.volume, 3_072_600.0);          // 手 → 股
        assert_eq!(k.quote_volume, 5_172_858_000.0);
        assert!(k.is_closed);
    }

    #[test]
    fn parse_negative_adjusted_bar() {
        // 前复权早期数据为负（茅台 2001 年实测），解析不得报错
        let k = parse_kline_line(
            "2001-08-27,-312.68,-312.47,-312.03,-313.01,406318,1410347000.00,-0.31,0.26,0.83,56.83",
            Interval::D1,
        ).unwrap();
        assert_eq!(k.open, -312.68);
        // OHLC 有序性对负价同样成立：high ≥ max(open, close)
        assert!(k.high >= k.open && k.high >= k.close);
        assert!(k.low <= k.open && k.low <= k.close);
    }

    #[test]
    fn parse_minute_bar() {
        let k = parse_kline_line(
            "2026-10-23 09:31,1311.50,1321.00,1323.65,1310.00,986,129903055.00",
            Interval::M5,
        ).unwrap();
        assert_eq!(k.close_time - k.open_time, 5 * 60 * 1000);
    }

    #[test]
    fn ohlc_order_invariant_over_full_history() {
        // 用 fixture 文件全量断言（见 1.7.2）
        let content = std::fs::read_to_string("tests/fixtures/eastmoney/kline_qfq.txt").unwrap();
        for line in content.lines() {
            let k = parse_kline_line(line, Interval::D1).unwrap();
            assert!(k.high >= k.open && k.high >= k.close, "high 破坏有序性: {line}");
            assert!(k.low <= k.open && k.low <= k.close, "low 破坏有序性: {line}");
        }
    }
}
```

### 1.7.2 fixture 文件（放 `stock-data/tests/fixtures/eastmoney/`）

| 文件                   | 内容来源                                              |
| ---------------------- | ----------------------------------------------------- |
| `ulist_quotes.json`    | §1.5.2 样例（林州重机/榕基软件两行）                  |
| `ulist_suspended.json` | 自造停牌行（f2/f3… 均为 `"-"`）→ 断言 `is_suspended`  |
| `clist_page1.json`     | 实测 clist 一页 + `"total":3488` → 断言分页终止条件   |
| `kline_qfq.txt`        | 茅台日线 CSV 若干行（含 2024 正价行 + 2001 负价行）   |
| `kline_null_data.json` | `{"data":null}` → 断言返回 `Err(Api)` 或空            |
| `trends.json`          | §1.5.4 样例 → 断言 preClose 与首点均价                |
| `quota_realtime.json`  | §1.8.4 实测 4 行（北向金额 0.0 占位 + 南向真值）      |
| `deal_new.json`        | §1.8.5 新口径行（2026-11-09 沪股通，金额字段全 null） |
| `deal_old.json`        | §1.8.5 旧口径行（2024-08-16 沪股通，净买额全套有值）  |

### 1.7.3 wiremock 集成测试（`stock-data/tests/http_tests.rs`）

```rust
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn quotes_parses_and_normalizes() {
    let server = MockServer::start().await;
    let fixture = std::fs::read_to_string("tests/fixtures/eastmoney/ulist_quotes.json").unwrap();
    Mock::given(method("GET"))
        .and(path("/api/qt/ulist.np/get"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
        .mount(&server)
        .await;

    // EastmoneyClient 需支持注入 base_url（测试构造函数），见下
    let client = stock_data::data::eastmoney::EastmoneyClient::with_base_url(server.uri()).unwrap();
    let quotes = client.quotes(&["002535"]).await.unwrap();

    assert_eq!(quotes.len(), 1);
    let q = &quotes[0];
    assert_eq!(q.code, "002535");
    assert_eq!(q.price, 3.03);
    assert_eq!(q.amount, 15_296.701_91);   // 元 → 万元 归一生效
    assert_eq!(q.total_mv, 24.290_997_14); // 元 → 亿元
}
```

> 为此 `EastmoneyClient` 要多一个 `with_base_url(base: String)` 构造函数：`BASE_Q`/`BASE_HIS` 从常量改为实例字段 `base_q`/`base_his`，生产构造 `new()` 填真实域名，测试构造填 mock 地址。这是注入 mock 的标准做法，也顺便解决"测试时被真实接口限速拖慢"的问题。

### 1.7.4 真实接口 smoke test（`#[ignore]`，手动跑）

```rust
#[tokio::test]
#[ignore = "需要真实网络，cargo test -- --ignored 手动执行"]
async fn smoke_kline_600519() {
    let client = EastmoneyClient::new().unwrap();
    let ks = client.klines("600519", Interval::D1, Adjust::Forward,
                           chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        .await.unwrap();
    assert!(!ks.is_empty());
    let k = ks.last().unwrap();
    assert!(k.high >= k.open && k.high >= k.close);
    assert!(k.low <= k.open && k.low <= k.close);
}

#[tokio::test]
#[ignore = "需要真实网络"]
async fn smoke_index_quotes() {
    let client = EastmoneyClient::new().unwrap();
    let idx = client.index_quotes().await.unwrap();
    assert_eq!(idx.len(), 8);
    assert!(idx.iter().all(|i| i.price > 0.0));
}
```

### 1.7.5 测试清单汇总

| 对象                 | 断言要点                                             | 位置                         |
| -------------------- | ---------------------------------------------------- | ---------------------------- |
| `secid()`            | 沪/深映射、北交拒绝、长度与字符校验                  | eastmoney.rs 单测            |
| `normalize`          | 元→万/亿、0/负值                                     | normalize.rs 单测            |
| `parse_kline_line`   | 字段顺序（收在高位前）、手→股、负价、分钟 close_time | eastmoney_types.rs 单测      |
| `parse_quote_row`    | 归一正确、停牌行 `is_suspended`                      | wiremock + fixture           |
| `phase()`            | §1.6.1 边界表 + 周末                                 | trading_hours.rs 单测        |
| 分页终止             | `total` 达到 / 空页 / `max_pages` 兜底               | wiremock（构造两页 fixture） |
| 空响应               | 200 + 空 body → `Err(Api)`（实测防风控回归）         | wiremock（空 body fixture）  |
| 真实接口             | OHLC 有序、指数 8 条                                 | `#[ignore]` smoke test       |
| `take_result_rows`   | success=false / result.data 缺失 → `Err(Api)`        | eastmoney_types 单测         |
| `north_turnover_pct` | 正常占比、分母 0 防护                                | eastmoney.rs 单测            |
| `parse_deal_row`     | 旧口径 `Some`、新口径 `None`、trim、枚举映射         | eastmoney_types 单测         |
| 北向方法             | wiremock 按 reportName 分路由，北向 2 / 南向 2       | wiremock                     |
| 真实接口（北向）     | QUOTA 4 条、北向成交额>0、近 20 日净买额全 None      | `#[ignore]` smoke test       |

---

## 1.8 北向资金数据（沪深港通）

> 2026-11-10 用 curl 实测（接口 A 当日实时 + 接口 B 历史日线 + 2024-08 断供过渡期）。JS 参考实现：stock-sdk `getNorthboundFlowSummary`（RPT_MUTUAL_QUOTA + quoteColumns 行情拼接）。

### 1.8.1 接口选型：两个报表分工

北向资金不走 push2/push2his，而是东财**数据中心**接口，域名完全不同：

```
https://datacenter-web.eastmoney.com/api/data/v1/get
```

| 报表（reportName）        | 用途         | 时效          | 核心字段                         |
| ------------------------- | ------------ | ------------- | -------------------------------- |
| `RPT_MUTUAL_QUOTA`        | 通道当日概览 | 盘中实时      | 标的池涨跌家数、净买额、关联指数 |
| `RPT_MUTUAL_DEAL_HISTORY` | 通道历史日线 | T+1（收盘后） | 成交总额、成交笔数、领涨股       |

**两个接口都接入**：QUOTA 供盘中实时卡片（涨跌家数/关联指数），DEAL_HISTORY 供成交总额主数字与历史走势。想只用一个接口同时拿到"当日实时 + 净买额"不存在——见 §1.8.2，净买额已经死了。

**通道枚举 MUTUAL_TYPE（两报表一致，实测确认）**

| code | 含义         | 方向 | 历史条数 | 领涨股样例 |
| ---- | ------------ | ---- | -------- | ---------- |
| 001  | 沪股通       | 北向 | 2785     | 603366.SH  |
| 002  | 港股通（沪） | 南向 | 2747     | 02335.HK   |
| 003  | 深股通       | 北向 | 2309     | 002037.SZ  |
| 004  | 港股通（深） | 南向 | 2280     | 02335.HK   |

> ⚠️ 巨坑：002 **不是**深股通，是港股通（沪）。网上老教程多把 001/002 当"沪/深股通"配对，照抄必拿错数据。**北向 = 001 + 003**。QUOTA 报表带 `FUNDS_DIRECTION:"北向"/"南向"` 可直接校验；DEAL_HISTORY 只能靠枚举表。

### 1.8.2 ⚠️ 2024-08-19 披露机制分水岭（必读）

沪深交易所自 **2024-08-19** 起调整沪深港通信息披露机制：北向（沪股通/深股通）**实时与日度净买额停止披露**，只保留成交总额、成交笔数等。实测铁证（沪股通 filter=MUTUAL_TYPE="001"）：

| 交易日     | FUND_INFLOW | NET_DEAL_AMT | BUY_AMT  | SELL_AMT | DEAL_AMT（万元） |
| ---------- | ----------- | ------------ | -------- | -------- | ---------------- |
| 2024-08-15 | 9629.62     | 8865.01      | 31345.92 | 22480.91 | 53826.83         |
| 2024-08-16 | -1786.23    | -2568.22     | 22080.14 | 24648.36 | 46728.50         |
| 2024-08-19 | null        | null         | null     | null     | 46129.34         |
| 2026-11-09 | null        | null         | null     | null     | 102043.18        |

结论：

- **历史净买额只到 2024-08-16 为止**，与新口径成交额不可直接比较或连续累加（官方口径）；
- 新口径唯一存活的金额字段是 `DEAL_AMT`（成交总额，万元），`DEAL_NUM`（笔数）照常；
- QUOTA 报表的北向行金额字段（`netBuyAmt`/`dayAmtRemain`/`dayNetAmtIn`）盘中返回 **0.0 占位**（status=3），不是 null——**显示"北向净流入 0.00 亿"就是造假**，UI 禁止；
- 南向（港股通）金额字段照常披露（`netBuyAmt` 实测 -135373.44 万），将来做南向面板不受影响。

### 1.8.3 响应壳与通用约定

datacenter 的壳与 push2 **不是一套**，`take_data()` 不适用：

```json
{
  "version": "e4caf...",
  "result": { "pages": 1, "data": [ ... ], "count": 4 },
  "success": true,
  "message": "ok",
  "code": 0
}
```

实测通用约定：

1. **无 Referer 也能通**（只带 UA 实测通过）——datacenter 不像 push2 风控那么紧，复用现有 client 请求头即可；
2. `columns` 支持裁剪（`columns=TRADE_DATE,DEAL_AMT` 实测有效），但 MVP 统一 `columns=ALL`，裁剪会让 null 语义与字段缺失混在一起；
3. `filter` 是完整表达式且**括号必须成对**：`filter=(MUTUAL_TYPE="001")` 编码为 `filter=%28MUTUAL_TYPE%3D%22001%22%29`，漏掉 `%29` 右括号返回 `参数预处理错误（ANTLR InputMismatchException）`；
4. `~` 与 `,` 在 URL query 中无需编码（quoteColumns 实测通过）；
5. 字符串字段普遍带前导空格（`"INDEX_NAME":" 深证成指"`、`"LEAD_STOCKS_NAME":" 力鼎光电"`），解析统一 `trim()`；
6. 日期统一 `"YYYY-MM-DD 00:00:00"`，截前 10 位 parse；
7. **单位**：金额万元；⚠️ `HOLD_MARKET_CAP` 是**元**（实测 1.18e12）；家数是"只"；涨跌幅是 `%`。

### 1.8.4 报表 A：RPT_MUTUAL_QUOTA（当日概览，实时）

**请求参数**：

| 参数                    | 值                                                                                                                                                                                                                 |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| reportName              | RPT_MUTUAL_QUOTA                                                                                                                                                                                                   |
| columns                 | TRADE_DATE,MUTUAL_TYPE,BOARD_TYPE,MUTUAL_TYPE_NAME,FUNDS_DIRECTION,INDEX_CODE,INDEX_NAME,BOARD_CODE                                                                                                                |
| quoteColumns            | status~07~BOARD_CODE,dayNetAmtIn~07~BOARD_CODE,dayAmtRemain~07~BOARD_CODE,dayAmtThreshold~07~BOARD_CODE,f104~07~BOARD_CODE,f105~07~BOARD_CODE,f106~07~BOARD_CODE,f3~03~INDEX_CODE~INDEX_f3,netBuyAmt~07~BOARD_CODE |
| quoteType               | 0                                                                                                                                                                                                                  |
| sortColumns / sortTypes | MUTUAL_TYPE / 1                                                                                                                                                                                                    |
| pageSize                | 2000（全表仅 4 行，一页拉完，**无需 filter**）                                                                                                                                                                     |

`quoteColumns` 语法：`行情字段~行情源~关联键[~别名]`——把行情系统的字段**拼接**进数据中心静态报表。`07` = 板块源（按 BOARD_CODE 关联），`03` = 指数源（按 INDEX_CODE 关联，`INDEX_f3` 是别名）。整段从 stock-sdk 原样抄，**不要手改**。

**返回字段**（实测 2026-11-10 盘中）：

| 字段                               | 含义                                                                                    | 单位 | 北向实测值                           |
| ---------------------------------- | --------------------------------------------------------------------------------------- | ---- | ------------------------------------ |
| TRADE_DATE                         | 当日                                                                                    | -    | "2026-11-10"（盘中即有今天）         |
| MUTUAL_TYPE / MUTUAL_TYPE_NAME     | 枚举 / 中文名                                                                           | -    | 001 / "沪股通"                       |
| FUNDS_DIRECTION                    | 方向                                                                                    | -    | "北向" / "南向"                      |
| BOARD_TYPE / BOARD_CODE            | "沪港通"/"深港通" / BK0707 等                                                           | -    |                                      |
| status                             | 实测南向=1、北向=3                                                                      | -    | 语义未确认，MVP 不使用               |
| f104 / f105 / f106                 | 标的池涨 / 跌 / 平家数                                                                  | 只   | 沪 1063/528/55；深 1433/404/56       |
| netBuyAmt                          | 当日净买额                                                                              | 万元 | **恒 0.0 占位**（南向有真值）        |
| dayAmtRemain                       | 额度剩余                                                                                | 万元 | 0.0（"额度充足"时为 0）              |
| dayNetAmtIn                        | ⚠️ 名义"当日资金流入"，南向实测 4200000.0 = 每日额度 420 亿，语义与名义不符，**不建模** | 万元 | 北向 0.0                             |
| dayAmtThreshold                    | 额度阈值（未逐字段实测）                                                                | 万元 | 北向预期恒 0                         |
| INDEX_CODE / INDEX_NAME / INDEX_f3 | 关联指数 / 实时涨跌幅                                                                   | %    | 000001 上证 / 399001 深成 / HSI 恒生 |

> ⚠️ f104/f105/f106 是**通道标的池**口径，不是全市场：沪池合计 1646 只、深池 1893 只、南向池 667 只（2026-11-10 实测）。UI 若拿它当全市场涨跌家数展示即是造假；全市场涨跌用 §1.5.6 market_snapshot。

**实测响应样例（北向行，可直接做 fixture）**：

```json
{
  "TRADE_DATE": "2026-11-10 00:00:00",
  "MUTUAL_TYPE": "001",
  "BOARD_TYPE": "沪港通",
  "MUTUAL_TYPE_NAME": "沪股通",
  "FUNDS_DIRECTION": "北向",
  "INDEX_CODE": "000001",
  "INDEX_NAME": "上证指数",
  "BOARD_CODE": "BK0707",
  "status": 3,
  "dayNetAmtIn": 0.0,
  "dayAmtRemain": 0.0,
  "f104": 1063,
  "f105": 528,
  "f106": 55,
  "INDEX_f3": 0.08,
  "netBuyAmt": 0.0
}
```

### 1.8.5 报表 B：RPT_MUTUAL_DEAL_HISTORY（历史日线，T+1）

**请求参数**：

| 参数                    | 值                                                     |
| ----------------------- | ------------------------------------------------------ |
| reportName              | RPT_MUTUAL_DEAL_HISTORY                                |
| columns                 | ALL                                                    |
| filter                  | %28MUTUAL_TYPE%3D%22001%22%29（沪股通；深股通 003）    |
| pageSize                | 想要几天传几天（≤2000），单页拉最近 N 条，**无需翻页** |
| pageNumber              | 1                                                      |
| sortColumns / sortTypes | TRADE_DATE / -1（降序，显式传）                        |

**字段全表**（沪股通实测）：

| 字段                                     | 含义                  | 单位   | 2024-08-16（旧口径） | 2026-11-09（新口径）   |
| ---------------------------------------- | --------------------- | ------ | -------------------- | ---------------------- |
| TRADE_DATE                               | 交易日                | -      | 有效                 | 有效（**最新为 T-1**） |
| FUND_INFLOW                              | 当日资金流入          | 万元   | -1786.23             | null                   |
| NET_DEAL_AMT                             | 当日净买额            | 万元   | -2568.22             | null                   |
| BUY_AMT / SELL_AMT                       | 买 / 卖成交额         | 万元   | 22080.14 / 24648.36  | null                   |
| QUOTA_BALANCE                            | 当日额度余额          | 万元   | 53786.23             | null                   |
| ACCUM_DEAL_AMT                           | 历史累计净流入        | 万元   | 1008120.64           | null                   |
| HOLD_MARKET_CAP                          | 持股市值              | **元** | 1181312830215.97     | null                   |
| DEAL_AMT                                 | 成交总额              | 万元   | 46728.50             | 102043.18 ✅           |
| DEAL_NUM                                 | 成交笔数              | 笔     | 3885742              | 5984738 ✅             |
| QUOTA_BALANCE_TEXT                       | 余额文本              | -      | "53786.23"（数字串） | "额度充足"（类型漂移） |
| LEAD_STOCKS_CODE / NAME / LS_CHANGE_RATE | 领涨股                | -      | 有效（北向 A 股）    | 有效                   |
| INDEX_CLOSE_PRICE / INDEX_CHANGE_RATE    | 关联指数收盘 / 涨跌幅 | -      | 有效                 | 有效                   |

**实测样例（新口径，2026-11-09 沪股通）**：

```json
{
  "MUTUAL_TYPE": "001",
  "TRADE_DATE": "2026-11-09 00:00:00",
  "FUND_INFLOW": null,
  "NET_DEAL_AMT": null,
  "QUOTA_BALANCE": null,
  "ACCUM_DEAL_AMT": null,
  "BUY_AMT": null,
  "SELL_AMT": null,
  "LEAD_STOCKS_CODE": "603366.SH",
  "LEAD_STOCKS_NAME": "日出东方",
  "LS_CHANGE_RATE": 10.01,
  "INDEX_CLOSE_PRICE": 3660.79,
  "INDEX_CHANGE_RATE": 0.32,
  "HOLD_MARKET_CAP": null,
  "DEAL_AMT": 102043.18,
  "QUOTA_BALANCE_TEXT": "额度充足",
  "DEAL_NUM": 5984738
}
```

### 1.8.6 模型设计（models.rs 新增）

```rust
/// 沪深港通通道（QUOTA / DEAL_HISTORY 两报表枚举一致，2026-11-10 实测）
/// ⚠️ 002 是港股通（沪）不是深股通；北向 = Sg(001) + Sz(003)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutualType {
    Sg,   // 001 沪股通（北向）
    HkSg, // 002 港股通（沪）（南向）
    Sz,   // 003 深股通（北向）
    HkSz, // 004 港股通（深）（南向）
}

impl MutualType {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "001" => Some(Self::Sg),
            "002" => Some(Self::HkSg),
            "003" => Some(Self::Sz),
            "004" => Some(Self::HkSz),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Sg => "001",
            Self::HkSg => "002",
            Self::Sz => "003",
            Self::HkSz => "004",
        }
    }

    /// 是否北向（外资经沪深港通买 A 股）
    pub fn is_north(self) -> bool {
        matches!(self, Self::Sg | Self::Sz)
    }
}

/// 通道当日概览（RPT_MUTUAL_QUOTA，盘中实时）
/// ⚠️ 北向行金额字段（net_buy_amt/remain_amt）为 0.0 占位（2024-08 停披露），
///    仅涨跌家数与关联指数实时有效；南向行金额照常
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NorthboundSummary {
    pub channel: MutualType,
    pub name: String,          // "沪股通"（响应带前导空格，解析已 trim）
    pub direction: String,     // "北向" / "南向"
    pub board: String,         // "沪港通" / "深港通"
    pub up_count: u32,         // f104 标的池上涨家数（通道池口径，非全市场）
    pub down_count: u32,       // f105
    pub flat_count: u32,       // f106
    pub net_buy_amt: f64,      // 万元；北向恒 0.0 占位，南向有效
    pub remain_amt: f64,       // 万元；额度剩余，"额度充足"时为 0
    pub index_code: String,    // 000001 / 399001 / HSI
    pub index_name: String,
    pub index_change_pct: f64, // %，实时（INDEX_f3）
    pub trade_date: chrono::NaiveDate, // QUOTA 盘中即返回当天
}

/// 通道历史日线（RPT_MUTUAL_DEAL_HISTORY，T+1 出数）
/// ⚠️ 2024-08-19 起净买额类字段 null → 解析为 None；禁用 0.0 兜底
///    （null 是"停披露"的契约语义，与"真实 0"必须可区分）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NorthboundDaily {
    pub channel: MutualType,
    pub date: chrono::NaiveDate,
    pub deal_amt: f64,                // 万元，成交总额（新旧口径唯一全存金额）
    pub deal_num: u64,                // 成交笔数
    pub net_deal_amt: Option<f64>,    // 万元，净买额（2024-08-19 起 None）
    pub fund_inflow: Option<f64>,     // 万元，当日资金流入
    pub buy_amt: Option<f64>,         // 万元，买入成交额
    pub sell_amt: Option<f64>,        // 万元，卖出成交额
    pub quota_balance: Option<f64>,   // 万元，当日额度余额
    pub hold_market_cap: Option<f64>, // ⚠️ 元（唯一"元"单位），持股市值
    pub lead_stock_code: String,      // 北向 "603366.SH" / 南向 "02335.HK"
    pub lead_stock_name: String,
    pub lead_stock_pct: f64,          // %，领涨股涨跌幅
    pub index_close: f64,             // 关联指数收盘（沪：上证，深：深成，南向：恒指）
    pub index_change_pct: f64,        // %
}
```

设计点：

- 金额用 `Option<f64>` 的都是"2024-08-19 后必为 null"的停披露字段；其余照旧 `f64_field` 归 0（笔数、指数价不存在 null 语义）；
- `MutualType` 用封闭枚举而不是 String：枚举封闭（001-004），`from_code` 返回 None 时该行整体跳过（`filter_map`），比"未知 code 硬塞字符串"安全；
- QUOTA 报表不建模 `dayNetAmtIn`/`dayAmtThreshold`/`status`：北向全 0 占位、南向语义存疑，MVP 用不上，少一个字段少一个坑。

### 1.8.7 解析层（eastmoney_types.rs 新增）

```rust
use crate::common::models::MutualType;   // 文件顶部补充

/// datacenter 壳：{version, result:{pages, data:[...], count}, success, message, code}
/// ⚠️ 与 push2 的 {rc, data} 是两套壳，take_data() 不适用
pub fn take_result_rows<'a>(v: &'a Value, api: &str) -> Result<Vec<&'a Value>> {
    if !v.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        return Err(DataError::Api(format!(
            "{api}: success=false, message={}",
            str_field(v, "message")
        )));
    }
    let arr = v
        .pointer("/result/data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| DataError::Api(format!("{api}: result.data 缺失")))?;
    Ok(arr.iter().collect())
}

/// 可缺失数值：null / 缺失 / 非数 → None
/// （与 f64_field 的"归 0"严格区分：专用于停披露字段，保住 null 语义）
pub fn f64_field_opt(v: &Value, key: &str) -> Option<f64> {
    match v.get(key) {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => s.parse::<f64>().ok(),
        _ => None,
    }
}

/// datacenter 日期 "2026-11-09 00:00:00" → NaiveDate（截前 10 位）
pub fn dc_date(v: &Value, key: &str) -> Option<chrono::NaiveDate> {
    str_field(v, key)
        .get(..10)
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
}

/// QUOTA 行 → NorthboundSummary；枚举未知 → None（调用方 filter_map 跳过）
pub fn parse_quota_row(row: &Value) -> Option<NorthboundSummary> {
    let channel = MutualType::from_code(str_field(row, "MUTUAL_TYPE"))?;
    Some(NorthboundSummary {
        channel,
        // 实测多个字符串字段带前导空格，统一 trim
        name: str_field(row, "MUTUAL_TYPE_NAME").trim().to_string(),
        direction: str_field(row, "FUNDS_DIRECTION").trim().to_string(),
        board: str_field(row, "BOARD_TYPE").to_string(),
        up_count: f64_field(row, "f104") as u32,
        down_count: f64_field(row, "f105") as u32,
        flat_count: f64_field(row, "f106") as u32,
        net_buy_amt: f64_field(row, "netBuyAmt"), // 北向 0.0 占位，语义由调用方判定
        remain_amt: f64_field(row, "dayAmtRemain"),
        index_code: str_field(row, "INDEX_CODE").trim().to_string(),
        index_name: str_field(row, "INDEX_NAME").trim().to_string(),
        index_change_pct: f64_field(row, "INDEX_f3"),
        trade_date: dc_date(row, "TRADE_DATE")?,
    })
}

/// DEAL_HISTORY 行 → NorthboundDaily；新口径 null 字段 → None
pub fn parse_deal_row(row: &Value) -> Option<NorthboundDaily> {
    let channel = MutualType::from_code(str_field(row, "MUTUAL_TYPE"))?;
    Some(NorthboundDaily {
        channel,
        date: dc_date(row, "TRADE_DATE")?,
        deal_amt: f64_field(row, "DEAL_AMT"),
        deal_num: f64_field(row, "DEAL_NUM") as u64,
        net_deal_amt: f64_field_opt(row, "NET_DEAL_AMT"),
        fund_inflow: f64_field_opt(row, "FUND_INFLOW"),
        buy_amt: f64_field_opt(row, "BUY_AMT"),
        sell_amt: f64_field_opt(row, "SELL_AMT"),
        quota_balance: f64_field_opt(row, "QUOTA_BALANCE"),
        hold_market_cap: f64_field_opt(row, "HOLD_MARKET_CAP"), // 元
        lead_stock_code: str_field(row, "LEAD_STOCKS_CODE").trim().to_string(),
        lead_stock_name: str_field(row, "LEAD_STOCKS_NAME").trim().to_string(),
        lead_stock_pct: f64_field(row, "LS_CHANGE_RATE"),
        index_close: f64_field(row, "INDEX_CLOSE_PRICE"),
        index_change_pct: f64_field(row, "INDEX_CHANGE_RATE"),
    })
}
```

### 1.8.8 客户端方法（eastmoney.rs 增量，以当前实际代码为基准）

> §1.4.1 骨架为初稿；当前实际代码已含 `cache: Arc<AnyTtlCach>` 字段。以下增量**以实际代码为基准**：

```rust
// ① 常量（放 UA 旁）
const BASE_DC: &str = "https://datacenter-web.eastmoney.com/api/data/v1/get";

// ② EastmoneyClient 新增字段 base_dc: String（与 base_q/base_his 同风格存完整路径前缀），
//    new() / with_base_url() 同步三参化：
pub fn new() -> Result<Self> {
    Self::with_base_url(
        "https://push2.eastmoney.com/api/qt".into(),
        "https://push2his.eastmoney.com/api/qt".into(),
        BASE_DC.into(),
    )
}
// ⚠️ 三参化后，现有 wiremock 测试的 with_base_url(...) 调用点必须同步补第三参

// ③ cache.rs 的 cache_prefix 新增：
pub const NORTHBOUND_SUMMARY: &str = "NB_SUMMARY"; // TTL 5s
pub const NORTHBOUND_HISTORY: &str = "NB_HIST";    // TTL 30min，key 拼 channel code

// ④ 数据方法
impl EastmoneyClient {
    /// 通道当日概览（实时）：无 filter 一次拉全 4 条（全表仅 4 行），
    /// 调用方按 MutualType::is_north() 筛北向
    pub async fn northbound_summary(&self) -> Result<Vec<NorthboundSummary>> {
        if let Some(hit) = self
            .cache
            .get::<Vec<NorthboundSummary>>(cache_prefix::NORTHBOUND_SUMMARY)
            .await
        {
            return Ok((*hit).clone());
        }
        const COLUMNS: &str = "TRADE_DATE,MUTUAL_TYPE,BOARD_TYPE,MUTUAL_TYPE_NAME,\
                               FUNDS_DIRECTION,INDEX_CODE,INDEX_NAME,BOARD_CODE";
        const QUOTE_COLUMNS: &str = "status~07~BOARD_CODE,dayNetAmtIn~07~BOARD_CODE,\
                                     dayAmtRemain~07~BOARD_CODE,dayAmtThreshold~07~BOARD_CODE,\
                                     f104~07~BOARD_CODE,f105~07~BOARD_CODE,f106~07~BOARD_CODE,\
                                     f3~03~INDEX_CODE~INDEX_f3,netBuyAmt~07~BOARD_CODE";
        let url = format!(
            "{}?reportName=RPT_MUTUAL_QUOTA&columns={}&quoteColumns={}&quoteType=0\
             &sortColumns=MUTUAL_TYPE&sortTypes=1&pageSize=2000&source=WEB&client=WEB",
            self.base_dc, COLUMNS, QUOTE_COLUMNS
        );
        let v = self.get_json(&url).await?;
        let rows: Vec<NorthboundSummary> = raw::take_result_rows(&v, "northbound-summary")?
            .iter()
            .filter_map(|r| raw::parse_quota_row(r))
            .collect();
        self.cache
            .insert(cache_prefix::NORTHBOUND_SUMMARY, rows.clone(), Duration::from_secs(5))
            .await;
        Ok(rows)
    }

    /// 通道历史日线（T+1）：单页 pageSize=days 拉最近 N 条，无需翻页
    pub async fn northbound_history(
        &self, channel: MutualType, days: usize,
    ) -> Result<Vec<NorthboundDaily>> {
        let key = format!("{}:{}", cache_prefix::NORTHBOUND_HISTORY, channel.code());
        if let Some(hit) = self.cache.get::<Vec<NorthboundDaily>>(&key).await {
            return Ok((*hit).clone());
        }
        // ⚠️ filter 括号必须成对；漏 %29 → "参数预处理错误"（实测踩过）
        let url = format!(
            "{}?reportName=RPT_MUTUAL_DEAL_HISTORY&columns=ALL&source=WEB&client=WEB\
             &filter=%28MUTUAL_TYPE%3D%22{}%22%29&pageSize={}&pageNumber=1\
             &sortColumns=TRADE_DATE&sortTypes=-1",
            self.base_dc,
            channel.code(),
            days.clamp(1, 2000),
        );
        let v = self.get_json(&url).await?;
        let rows: Vec<NorthboundDaily> = raw::take_result_rows(&v, "northbound-history")?
            .iter()
            .filter_map(|r| raw::parse_deal_row(r))
            .collect();
        self.cache
            .insert(&key, rows.clone(), Duration::from_secs(30 * 60))
            .await;
        Ok(rows)
    }
}
```

### 1.8.9 缓存、轮询与 UI 卡片对接口径

TTL：见 §1.6.2 新增的 `NB_SUMMARY`（5s）/ `NB_HIST:{channel}`（30min）两行。

轮询策略：

- summary 并入现有 quotes 轮询 fetch（盘中 5s），失败走既有退避，不发默认值（一次性请求发空值吞错的教训见 memory/03 集成文档）；
- history 启动拉一次 + 收盘后（Afternoon→Closed 切换）重拉一次即可；MVP 甚至可以只在启动时拉。

**总览页北向卡片字段口径**（替代当前硬编码 "0.54亿"；2026-11-10 定稿：主指标采用**北上成交占比**——净买额停披露后外资相对活跃度的标准替代口径）：

| 卡片位置   | 数据                                                                | 来源                      |
| ---------- | ------------------------------------------------------------------- | ------------------------- |
| 主数字     | 北上成交占比（%）= 北上成交总额 / 两市成交额 × 100                  | `north_turnover_pct`      |
| 主数字注脚 | "北上成交 XX.X 亿 · 2026-11-09"（绝对值放注脚不丢，日期 = **T-1**） | northbound_history(Sg/Sz) |
| 副行       | 上涨 X / 下跌 Y = 沪 f104+深 f104 / f105+f105（实时）               | northbound_summary        |
| 副行注脚   | "北向标的池口径"                                                    | -                         |

**占比分母口径（两档）**——分子恒为 T-1 北上成交总额（沪 + 深 `deal_amt`，万元）：

| 档位 | 分母                                                                                        | 口径一致性                                                           | 依赖           |
| ---- | ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | -------------- |
| MVP  | `MarketSnapshot.total_amount`（实时，亿元 × 1e4）                                           | 混搭：T-1 分子 × 今日分母，盘中占比偏低，**收盘后自动变准**，UI 注明 | 已有，零新依赖 |
| 精确 | 上证指数 `1.000001` + 深证综指 `0.399106` 昨日 D1 K 线 `quote_volume`（元，/ 1e4 对齐万元） | T-1 分子 × T-1 分母，全自洽                                          | 需 klines 实装 |

> ⚠️ 深市全口径是**深证综指 399106**，不是 `INDEX_SECIDS` 里现成的深证成指 399001——后者只有 500 只成分股，拿它做分母占比直接虚高约一倍。指数走显式 secid 常量，不经 `secid()` 规则函数（39 开头会被拒绝）。近似误差：上证指数含 B 股成交而分子是 A 股标的池，占比略偏低，MVP 接受。

占比计算做成纯函数（放 eastmoney.rs 底部，与 `secid()` 同待遇），client 只出归一数据，组装在 UI/service：

```rust
/// 北上成交占比（%）：分子北上成交总额（万元），分母两市成交额（万元）
/// MVP 分母 = MarketSnapshot.total_amount × 1e4；精确版 = 沪深综指昨日 K 线 quote_volume（元）/ 1e4
pub fn north_turnover_pct(north_wan: f64, market_wan: f64) -> f64 {
    if market_wan <= 0.0 { return 0.0; }
    north_wan / market_wan * 100.0
}
```

组装路径（MVP）：`northbound_history(Sg, 1)` + `northbound_history(Sz, 1)` 取最新两行 `deal_amt` 之和（万元），分母 `market_snapshot().total_amount × 1e4`（万元），喂给 `north_turnover_pct`。

⚠️ 禁止显示"北向净流入/净买额"：数据 2024-08-19 已停披露，接口返回 0.0 占位，照显示就是造假。想看外资动向只能看成交总额/成交占比与标的池涨跌家数；南向净买额可以正常显示（数据照常披露）。

### 1.8.10 测试方案与 fixture

```rust
// eastmoney_types.rs 底部
#[cfg(test)]
mod northbound_tests {
    use super::*;
    use crate::common::models::MutualType;

    #[test]
    fn mutual_type_codes() {
        assert_eq!(MutualType::from_code("001"), Some(MutualType::Sg));
        assert_eq!(MutualType::from_code("003"), Some(MutualType::Sz));
        assert_eq!(MutualType::from_code("002"), Some(MutualType::HkSg));
        assert_eq!(MutualType::from_code("999"), None);
        assert!(MutualType::Sg.is_north() && MutualType::Sz.is_north());
        assert!(!MutualType::HkSg.is_north() && !MutualType::HkSz.is_north());
    }

    #[test]
    fn take_result_rows_rejects_failure_and_missing() {
        let bad: Value = serde_json::from_str(
            r#"{"success":false,"message":"参数错误","code":1,"result":null}"#,
        ).unwrap();
        assert!(matches!(take_result_rows(&bad, "t"), Err(DataError::Api(_))));
        let empty: Value = serde_json::from_str(
            r#"{"success":true,"result":{"pages":1}}"#,
        ).unwrap();
        assert!(matches!(take_result_rows(&empty, "t"), Err(DataError::Api(_))));
    }

    #[test]
    fn parse_quota_north_row() {
        // §1.8.4 实测北向行：金额 0.0 占位，家数/指数有效，名称带空格
        let row: Value = serde_json::from_str(r#"{
            "TRADE_DATE":"2026-11-10 00:00:00","MUTUAL_TYPE":"001","BOARD_TYPE":"沪港通",
            "MUTUAL_TYPE_NAME":"沪股通","FUNDS_DIRECTION":"北向","INDEX_CODE":"000001",
            "INDEX_NAME":"上证指数","BOARD_CODE":"BK0707","status":3,
            "dayNetAmtIn":0.0,"dayAmtRemain":0.0,"f104":1063,"f105":528,"f106":55,
            "INDEX_f3":0.08,"netBuyAmt":0.0}"#).unwrap();
        let s = parse_quota_row(&row).unwrap();
        assert_eq!(s.channel, MutualType::Sg);
        assert!(s.channel.is_north());
        assert_eq!((s.up_count, s.down_count, s.flat_count), (1063, 528, 55));
        assert_eq!(s.net_buy_amt, 0.0);      // 占位值原样保留
        assert_eq!(s.trade_date.to_string(), "2026-11-10");
    }

    #[test]
    fn parse_deal_row_old_vs_new() {
        // 旧口径（2024-08-16 实测）：净买额全套 Some
        let old: Value = serde_json::from_str(r#"{
            "MUTUAL_TYPE":"001","TRADE_DATE":"2024-08-16 00:00:00",
            "FUND_INFLOW":-1786.23,"NET_DEAL_AMT":-2568.22,"QUOTA_BALANCE":53786.23,
            "BUY_AMT":22080.14,"SELL_AMT":24648.36,"HOLD_MARKET_CAP":1181312830215.97,
            "DEAL_AMT":46728.5,"DEAL_NUM":3885742,
            "LEAD_STOCKS_CODE":"605118.SH","LEAD_STOCKS_NAME":" 力鼎光电",
            "LS_CHANGE_RATE":10.01,"INDEX_CLOSE_PRICE":2879.43,"INDEX_CHANGE_RATE":0.07}"#).unwrap();
        let d = parse_deal_row(&old).unwrap();
        assert_eq!(d.net_deal_amt, Some(-2568.22));
        assert_eq!(d.hold_market_cap, Some(1181312830215.97)); // 元，勿换算
        assert_eq!(d.lead_stock_name, "力鼎光电");             // trim 生效

        // 新口径（2026-11-09 实测）：停披露字段 None，成交总额存活
        let new: Value = serde_json::from_str(r#"{
            "MUTUAL_TYPE":"001","TRADE_DATE":"2026-11-09 00:00:00","FUND_INFLOW":null,
            "NET_DEAL_AMT":null,"QUOTA_BALANCE":null,"ACCUM_DEAL_AMT":null,
            "BUY_AMT":null,"SELL_AMT":null,"HOLD_MARKET_CAP":null,
            "DEAL_AMT":102043.18,"DEAL_NUM":5984738,
            "LEAD_STOCKS_CODE":"603366.SH","LEAD_STOCKS_NAME":"日出东方",
            "LS_CHANGE_RATE":10.01,"INDEX_CLOSE_PRICE":3660.79,"INDEX_CHANGE_RATE":0.32}"#).unwrap();
        let d = parse_deal_row(&new).unwrap();
        assert_eq!(d.net_deal_amt, None);
        assert_eq!(d.deal_amt, 102043.18);
        assert_eq!(d.deal_num, 5984738);
    }
}
```

```rust
// tests/http_tests.rs 追加（wiremock）
#[tokio::test]
async fn northbound_summary_parses_all_four_channels() {
    let server = MockServer::start().await;
    let fixture = std::fs::read_to_string("tests/fixtures/eastmoney/quota_realtime.json").unwrap();
    Mock::given(method("GET"))
        .and(path("/api/data/v1/get"))
        .and(query_param("reportName", "RPT_MUTUAL_QUOTA"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
        .mount(&server)
        .await;
    // with_base_url 三参化后：第三参指向 mock 的 datacenter 路径
    let client = EastmoneyClient::with_base_url(
        "http://unused".into(),
        "http://unused".into(),
        format!("{}/api/data/v1/get", server.uri()),
    ).unwrap();
    let rows = client.northbound_summary().await.unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows.iter().filter(|r| r.channel.is_north()).count(), 2);
}

#[tokio::test]
#[ignore = "需要真实网络"]
async fn smoke_northbound() {
    let client = EastmoneyClient::new().unwrap();
    let rows = client.northbound_summary().await.unwrap();
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().filter(|r| r.channel.is_north())
        .all(|r| r.up_count + r.down_count + r.flat_count > 0));

    let hist = client.northbound_history(MutualType::Sg, 20).await.unwrap();
    assert!(!hist.is_empty());
    let latest = &hist[0];               // sortTypes=-1 首条即最新
    assert!(latest.deal_amt > 0.0 && latest.deal_num > 0);
    // 2024-08-19 停披露：近 20 日净买额全 None 是预期
    assert!(hist.iter().all(|d| d.net_deal_amt.is_none()));
}
```

```rust
// eastmoney.rs 底部 tests mod 追加
#[test]
fn north_turnover_pct_rules() {
    assert!((north_turnover_pct(15_000.0, 1_000_000.0) - 1.5).abs() < 1e-9);
    assert_eq!(north_turnover_pct(15_000.0, 0.0), 0.0);   // 分母 0 防护（接口异常/空市）
    assert_eq!(north_turnover_pct(0.0, 1_000_000.0), 0.0);
}
```

### 1.8.11 踩坑清单（实测记录）

1. **MUTUAL_TYPE 直觉陷阱**：002 是港股通（沪）不是深股通；老教程按 001=沪、002=深 配对，照抄拿错向。北向 = 001+003，QUOTA 报表可用 FUNDS_DIRECTION 兜底校验。
2. **2024-08-19 分水岭**：北向净买额停披露。QUOTA 北向金额 0.0 占位（status=3），DEAL_HISTORY 金额 null。null 用 `Option`、0.0 占位原样保留并注释，别混。
3. **filter 括号必须成对**：`(MUTUAL_TYPE="001")` 编码 `%28...%29`，漏 `%29` → `参数预处理错误（ANTLR InputMismatchException）`（实测踩过）。
4. **单位混装**：金额万元、HOLD_MARKET_CAP 元、家数只、涨跌幅 %。成交总额万元 → UI 亿：除以 1e4。
5. **字符串前导空格**：" 深证成指"、" 力鼎光电"、" 额度充足"，统一 trim。
6. **DEAL_HISTORY 无当日行**：盘中查最新行 = T-1；UI 显示必须带数据日期，否则用户以为是今天。
7. **quoteColumns 整段照抄**：语法 `字段~行情源~关联键[~别名]`，自行增删容易断链（`07` 源按 BOARD_CODE 关联、`03` 源按 INDEX_CODE 关联）。
8. **f104/f105/f106 是通道标的池口径**（沪池 1646、深池 1893、南向池 667），不是全市场涨跌；全市场口径用 market_snapshot。
9. **占比分母别拿 399001 深证成指**：500 只成分股指数不是深市全口径，深市全口径是深证综指 399106；MVP 混搭口径（T-1 分子 × 今日分母）盘中偏低属预期，UI 注明"收盘后为准"。
