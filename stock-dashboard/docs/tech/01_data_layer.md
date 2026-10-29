# Phase 1：数据层技术实现

> 所有接口规格均于 **2026-10-23 用 curl 实测验证**（贵州茅台 600519、上证指数等）。实现时如遇字段不符，以实测为准并回写本文。

## 1.1 模块概述

### 目标

接入东方财富公开行情接口，提供**批量快照（自选/指数）、全市场列表、历史 K 线、当日分时**四类数据，配 TTL 缓存与轮询调度，为 UI 层提供唯一数据出口。

### 模块划分与职责

| 文件 | 职责 | 关键类型/函数 | 对应任务 |
|------|------|--------------|---------|
| `common/error.rs` | 统一错误类型 | `DataError`、`Result<T>` | T1.2 |
| `common/models.rs` | 归一后的领域模型 | `Quote` `IndexQuote` `Kline` `TimelinePoint` `MarketSnapshot` `Interval` `Adjust` | T1.2 |
| `common/trading_hours.rs` | 交易时段判断（纯函数） | `TradingPhase`、`phase()`、`now_shanghai()` | T3.1（定义在 Phase 1 建） |
| `data/eastmoney_types.rs` | 东财原始响应解析 | `take_data()`、`f64_field()` 等 helper、`parse_quote_row()`、kline/分时 CSV 解析 | T1.4 |
| `data/normalize.rs` | 单位归一（元→万/亿、手→股） | `yuan_to_wan()` 等 | T1.4 |
| `data/eastmoney.rs` | HTTP 客户端 | `EastmoneyClient`、`TokenBucket`、`secid()` | T1.3/T1.4 |
| `data/cache.rs` | 内存 TTL 缓存 | `TtlCache<V>` | T1.5 |
| `data/polling.rs` | 轮询调度（时段感知 + 退避） | `spawn_polling()`、`PollingHandle` | T3.1 |

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
{"rc":0,"data":{"total":3488,"diff":[
  {"f2":3.03,"f3":10.18,"f5":505496,"f6":152967019.1,"f8":6.77,
   "f12":"002535","f13":0,"f14":"林州重机","f15":3.03,"f16":3.0,
   "f17":3.01,"f18":2.75,"f20":2429099714,"f21":2261402731}]}}
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
{"data":{"code":"600519","market":1,"name":"贵州茅台","dktotal":6027,
 "klines":["2024-01-02,1685.00,1685.90,1698.00,1670.11,30726,5172858000.00,1.72,-0.33,-5.62,0.24",
           "2001-08-27,-312.68,-312.47,-312.03,-313.01,406318,1410347000.00,-0.31,0.26,0.83,56.83"]}}
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
{"data":{"code":"600519","preClose":1318.49,
 "trends":["2026-10-23 09:30,1311.84,1311.84,1311.84,1311.84,1049,137612016.00,1311.840",
           "2026-10-23 09:31,1311.50,1321.00,1323.65,1310.00,986,129903055.00,1314.441"]}}
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
    /// 实测全市场约 5400 只，pz=100 约 55 页；max_pages 兜底防死循环
    pub async fn market_snapshot(&self) -> Result<MarketSnapshot> {
        let fs = "m:1+t:2,m:0+t:6,m:0+t:80,m:1+t:23"; // 沪深主板+创业+科创，不含北交
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

        Ok(MarketSnapshot {
            total: rows.len(),
            up_count: rows.iter().filter(|q| q.change_pct > 0.0).count(),
            down_count: rows.iter().filter(|q| q.change_pct < 0.0).count(),
            flat_count: rows.iter().filter(|q| q.change_pct == 0.0).count(),
            // 近似口径：统一按 ≥9.9% 计（创业板/科创 20cm 会被高估，MVP 接受，UI 注明）
            limit_up: rows.iter().filter(|q| q.change_pct >= 9.9).count(),
            limit_down: rows.iter().filter(|q| q.change_pct <= -9.9).count(),
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

> `market_snapshot` 的 `total_amount`：`q.amount` 已是万元，×5400 只 ≈ 5.4e7 万元，除以 1e4 转回亿元，注意别二次换算。

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

| key 前缀 | TTL |
|----------|-----|
| `ulist:{secids}` | 5s |
| `kline:{secid}:{klt}:{fqt}:{beg}` | 10min |
| `clist:snapshot` | 60s |

### 1.6.3 polling.rs — 轮询服务（Phase 1 建骨架，Phase 3 接 UI）

```rust
// src/data/polling.rs
use crate::common::trading_hours::{self, TradingPhase};
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

/// 启动轮询任务。
/// `on_quotes` 在 tokio 线程被调用，Phase 3 中它内部做 cx.update + cx.notify。
/// 失败退避：1s→2s→4s→…→30s 封顶，成功后恢复时段标准间隔。
pub fn spawn_polling<F>(client: Arc<crate::data::eastmoney::EastmoneyClient>, on_quotes: F) -> PollingHandle
where
    F: Fn(Vec<crate::common::models::Quote>) + Send + 'static,
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

            // TODO(Phase 3)：这里换成 watchlist 的 codes
            match client.quotes(&["600519", "000001"]).await {
                Ok(qs) => { backoff_secs = 0; on_quotes(qs); }
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

| 文件 | 内容来源 |
|------|---------|
| `ulist_quotes.json` | §1.5.2 样例（林州重机/榕基软件两行） |
| `ulist_suspended.json` | 自造停牌行（f2/f3… 均为 `"-"`）→ 断言 `is_suspended` |
| `clist_page1.json` | 实测 clist 一页 + `"total":3488` → 断言分页终止条件 |
| `kline_qfq.txt` | 茅台日线 CSV 若干行（含 2024 正价行 + 2001 负价行） |
| `kline_null_data.json` | `{"data":null}` → 断言返回 `Err(Api)` 或空 |
| `trends.json` | §1.5.4 样例 → 断言 preClose 与首点均价 |

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

| 对象 | 断言要点 | 位置 |
|------|---------|------|
| `secid()` | 沪/深映射、北交拒绝、长度与字符校验 | eastmoney.rs 单测 |
| `normalize` | 元→万/亿、0/负值 | normalize.rs 单测 |
| `parse_kline_line` | 字段顺序（收在高位前）、手→股、负价、分钟 close_time | eastmoney_types.rs 单测 |
| `parse_quote_row` | 归一正确、停牌行 `is_suspended` | wiremock + fixture |
| `phase()` | §1.6.1 边界表 + 周末 | trading_hours.rs 单测 |
| 分页终止 | `total` 达到 / 空页 / `max_pages` 兜底 | wiremock（构造两页 fixture） |
| 空响应 | 200 + 空 body → `Err(Api)`（实测防风控回归） | wiremock（空 body fixture） |
| 真实接口 | OHLC 有序、指数 8 条 | `#[ignore]` smoke test |
