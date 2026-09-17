# Phase 1：数据层技术实现

## 1.1 模块概述

### 目标

接入 Binance REST API，获取历史 K 线数据并本地持久化，为上层指标计算和 UI 展示提供数据基础。

### 覆盖的 Rust 知识点

| 知识点 | 实践场景 |
|--------|---------|
| **async/await** | 异步 HTTP 请求、tokio runtime |
| **reqwest** | REST API 调用、查询参数构建 |
| **serde / serde_json** | JSON 响应反序列化、自定义 Deserialize |
| **polars** | K 线 DataFrame 构建、Parquet 文件读写 |
| **tokio** | 异步任务调度、runtime 配置 |
| **chrono** | Unix 毫秒时间戳与 DateTime 互转 |
| **thiserror** | 库层错误类型定义 |
| **tracing** | 结构化日志（数据获取链路追踪） |

### 模块在项目中的位置

```
common/          ← 本阶段新增：错误类型、数据模型、配置
data/            ← 本阶段新增：API 客户端、数据缓存、数据服务
```

依赖方向：`data → common`，上层模块（indicators、ui）后续依赖 `data`。

---

## 1.2 项目初始化

### Cargo.toml 依赖配置（Phase 1 涉及的 crate）

```toml
[package]
name = "rustquant"
version = "0.1.0"
edition = "2021"

[dependencies]
# 异步运行时
tokio = { version = "1", features = ["full"] }

# HTTP 客户端
reqwest = { version = "0.12", features = ["json"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 数据处理
polars = { version = "0.46", features = ["lazy", "temporal", "parquet"] }

# 错误处理
anyhow = "1"
thiserror = "2"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 本阶段项目目录结构

```
src/
├── common/
│   ├── mod.rs           # 公共模块入口
│   ├── error.rs         # QuantError 统一错误类型
│   ├── config.rs        # 全局配置（API 地址、默认参数）
│   └── models.rs        # Kline、Interval 等核心数据模型
├── data/
│   ├── mod.rs           # 数据层入口
│   ├── fetcher.rs       # BinanceClient — REST API 数据获取
│   ├── kline_store.rs   # KlineStore — 本地 Parquet 缓存
│   └── types.rs         # Binance API 原始响应类型（serde 反序列化）
├── main.rs              # 程序入口
└── lib.rs               # 库入口、模块声明
```

---

## 1.3 数据模型定义

### Kline 结构体

```rust
// src/common/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// K 线数据（OHLCV）
/// 对应 Binance GET /api/v3/klines 返回的单根 K 线
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Kline {
    /// 开盘时间（Unix 毫秒时间戳）
    pub open_time: i64,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量（基础资产数量）
    pub volume: f64,
    /// 收盘时间（Unix 毫秒时间戳）
    pub close_time: i64,
    /// 成交额（报价资产数量，如 USDT）
    pub quote_volume: f64,
    /// 成交笔数
    pub trades_count: u32,
    /// 是否已收盘（K 线是否已完成）
    pub is_closed: bool,
}

impl Kline {
    /// 将 open_time 毫秒时间戳转为 DateTime<Utc>
    pub fn open_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.open_time)
            .expect("invalid open_time timestamp")
    }
}
```

### KlineInterval 枚举

```rust
// src/common/models.rs

/// K 线周期枚举
/// 与 Binance API 的 interval 参数一一对应
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Interval {
    M1,   // 1 分钟
    M5,   // 5 分钟
    M15,  // 15 分钟
    H1,   // 1 小时
    H4,   // 4 小时
    D1,   // 1 天
    W1,   // 1 周
}

impl Interval {
    /// 转为 Binance API 的 interval 字符串参数
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::M1  => "1m",
            Interval::M5  => "5m",
            Interval::M15 => "15m",
            Interval::H1  => "1h",
            Interval::H4  => "4h",
            Interval::D1  => "1d",
            Interval::W1  => "1w",
        }
    }

    /// 返回该周期的毫秒数（用于增量更新计算时间偏移）
    pub fn as_millis(&self) -> i64 {
        match self {
            Interval::M1  => 60_000,
            Interval::M5  => 300_000,
            Interval::M15 => 900_000,
            Interval::H1  => 3_600_000,
            Interval::H4  => 14_400_000,
            Interval::D1  => 86_400_000,
            Interval::W1  => 604_800_000,
        }
    }
}
```

### ExchangeInfo 结构体

```rust
// src/common/models.rs

/// 交易对信息（来自 Binance GET /api/v3/exchangeInfo）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    /// 交易对名称，如 "BTCUSDT"
    pub symbol: String,
    /// 交易对状态（"TRADING" 表示正常交易）
    pub status: String,
    /// 基础资产精度（如 BTC 的数量小数位数）
    pub base_asset_precision: u8,
    /// 报价资产精度（如 USDT 的价格小数位数）
    pub quote_asset_precision: u8,
}
```

### 设计要点

- `Kline` 使用 `f64` 存储价格，Binance 返回字符串数字，在反序列化阶段完成转换
- `Interval` 提供 `as_millis()` 方法，用于增量更新时计算下一根 K 线的起始时间
- `is_closed` 字段区分已完成 K 线和当前进行中的 K 线

---

## 1.4 Binance REST API 客户端

### BinanceClient 结构体

```rust
// src/data/fetcher.rs

use reqwest::Client;
use crate::common::models::Interval;
use crate::common::error::QuantError;
use crate::data::types::RawKlineResponse;

/// Binance REST API 客户端
/// 封装 reqwest::Client，提供 K 线数据获取能力
pub struct BinanceClient {
    /// HTTP 客户端（复用连接池）
    client: Client,
    /// API 基础地址
    base_url: String,
}

impl BinanceClient {
    /// 创建新的客户端实例
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    /// 获取历史 K 线数据
    ///
    /// # 参数
    /// - `symbol`: 交易对名称，如 "BTCUSDT"
    /// - `interval`: K 线周期
    /// - `start_time`: 起始时间（Unix 毫秒时间戳），可选
    /// - `limit`: 返回数量上限（Binance 最大 1000）
    pub async fn fetch_klines(
        &self,
        symbol: &str,
        interval: Interval,
        start_time: Option<i64>,
        limit: u32,
    ) -> Result<Vec<crate::common::models::Kline>, QuantError> {
        // 构建请求 URL 和查询参数
        let url = format!("{}/api/v3/klines", self.base_url);
        let mut request = self.client.get(&url)
            .query(&[("symbol", symbol), ("interval", interval.as_str())])
            .query(&[("limit", limit.to_string())]);

        if let Some(st) = start_time {
            request = request.query(&[("startTime", st.to_string())]);
        }

        // 发送请求
        let response = request.send().await
            .map_err(|e| QuantError::Network(e.to_string()))?;

        // 检查 HTTP 状态码
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(QuantError::Api(format!(
                "Binance API 错误: status={}, body={}", status, body
            )));
        }

        // 解析 JSON 响应
        let raw: RawKlineResponse = response.json().await
            .map_err(|e| QuantError::Parse(e.to_string()))?;

        // 将原始数据转换为 Kline 结构体
        raw.into_klines()
    }

    /// 获取交易对信息
    pub async fn fetch_exchange_info(
        &self,
    ) -> Result<Vec<crate::common::models::ExchangeInfo>, QuantError> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);
        let response = self.client.get(&url).send().await
            .map_err(|e| QuantError::Network(e.to_string()))?;

        // 解析并提取交易对列表
        // ... 省略完整实现
        todo!()
    }
}
```

### API 响应解析

```rust
// src/data/types.rs

use serde::Deserialize;
use crate::common::models::Kline;
use crate::common::error::QuantError;

/// Binance K 线 API 原始响应
/// Binance 返回格式：[[openTime, open, high, low, close, volume, closeTime, ...], ...]
/// 每个 K 线是一个 JSON 数组，元素为字符串或数字
#[derive(Debug, Deserialize)]
pub struct RawKlineResponse(Vec<RawKlineRow>);

/// 单根 K 线的原始数据（Binance 数组格式）
#[derive(Debug, Deserialize)]
pub struct RawKlineRow(
    pub i64,     // 0: open_time
    pub String,  // 1: open
    pub String,  // 2: high
    pub String,  // 3: low
    pub String,  // 4: close
    pub String,  // 5: volume
    pub i64,     // 6: close_time
    pub String,  // 7: quote_volume
    pub u32,     // 8: trades_count
    pub String,  // 9: taker_buy_volume（忽略）
    pub String,  // 10: taker_buy_quote_volume（忽略）
    pub String,  // 11: ignore（忽略）
);

impl RawKlineResponse {
    /// 将原始响应转换为 Kline 结构体列表
    pub fn into_klines(self) -> Result<Vec<Kline>, QuantError> {
        self.0.into_iter().map(|row| {
            Ok(Kline {
                open_time: row.0,
                open: row.1.parse().map_err(|_| QuantError::Parse("open price".into()))?,
                high: row.2.parse().map_err(|_| QuantError::Parse("high price".into()))?,
                low: row.3.parse().map_err(|_| QuantError::Parse("low price".into()))?,
                close: row.4.parse().map_err(|_| QuantError::Parse("close price".into()))?,
                volume: row.5.parse().map_err(|_| QuantError::Parse("volume".into()))?,
                close_time: row.6,
                quote_volume: row.7.parse().map_err(|_| QuantError::Parse("quote_volume".into()))?,
                trades_count: row.8,
                is_closed: true, // 历史 K 线默认已收盘
            })
        }).collect()
    }
}
```

### 设计要点

- `BinanceClient` 持有 `reqwest::Client`（内部连接池复用），不每次请求重建
- Binance K 线响应是嵌套数组格式，用 tuple struct `RawKlineRow` 按位置反序列化
- 价格字段以 `String` 接收再手动 `parse::<f64>()`，避免浮点精度丢失
- `fetch_klines` 的 `start_time` 参数可选，用于增量更新时指定起始位置
- 错误分为三类：`Network`（连接失败）、`Api`（HTTP 非 2xx）、`Parse`（JSON/字段解析失败）

---

## 1.5 数据持久化

### KlineStore 结构体

```rust
// src/data/kline_store.rs

use std::path::{Path, PathBuf};
use polars::prelude::*;
use crate::common::models::{Kline, Interval};
use crate::common::error::QuantError;

/// K 线数据本地缓存
/// 使用 Parquet 文件格式，利用 polars 的高效列式 IO
pub struct KlineStore {
    /// 缓存根目录（如 ./data/klines/）
    cache_dir: PathBuf,
}

impl KlineStore {
    /// 创建新的存储实例，自动创建缓存目录
    pub fn new(cache_dir: &Path) -> Result<Self, QuantError> {
        std::fs::create_dir_all(cache_dir)
            .map_err(|e| QuantError::Io(e.to_string()))?;
        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
        })
    }

    /// 生成缓存文件路径：{cache_dir}/{symbol}/{interval}.parquet
    fn file_path(&self, symbol: &str, interval: Interval) -> PathBuf {
        self.cache_dir
            .join(symbol.to_lowercase())
            .join(format!("{}.parquet", interval.as_str()))
    }

    /// 保存 K 线数据到 Parquet 文件
    /// 如果文件已存在，则合并新旧数据（按 open_time 去重）
    pub fn save_klines(
        &self,
        symbol: &str,
        interval: Interval,
        klines: &[Kline],
    ) -> Result<(), QuantError> {
        if klines.is_empty() {
            return Ok(());
        }

        // 将 Kline 切片转为 polars DataFrame
        let df = klines_to_dataframe(klines)?;

        // 如果已有缓存文件，加载并合并
        let path = self.file_path(symbol, interval);
        let merged = if path.exists() {
            let existing = self.load_klines(symbol, interval)?;
            let existing_df = klines_to_dataframe(&existing)?;
            // 纵向拼接后按 open_time 去重
            polars::functions::concat(&[existing_df, df], Vertical)?
                .unique(None, UniqueKeepStrategy::First, None)?
        } else {
            df
        };

        // 按 open_time 排序后写入 Parquet
        let sorted = merged.sort(["open_time"], SortMultipleOptions::default())?;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| QuantError::Io(e.to_string()))?;
        }

        let file = std::fs::File::create(&path)
            .map_err(|e| QuantError::Io(e.to_string()))?;
        ParquetWriter::new(file).finish(&sorted)?;

        tracing::info!(
            symbol = symbol,
            interval = interval.as_str(),
            count = sorted.height(),
            "K 线数据已保存"
        );

        Ok(())
    }

    /// 从 Parquet 文件加载 K 线数据
    pub fn load_klines(
        &self,
        symbol: &str,
        interval: Interval,
    ) -> Result<Vec<Kline>, QuantError> {
        let path = self.file_path(symbol, interval);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = std::fs::File::open(&path)
            .map_err(|e| QuantError::Io(e.to_string()))?;
        let df = ParquetReader::new(file).finish()?;

        Ok(dataframe_to_klines(&df)?)
    }

    /// 查询本地缓存的数据范围（最早/最晚时间戳）
    /// 用于增量更新时判断需要获取的数据区间
    pub fn get_data_range(
        &self,
        symbol: &str,
        interval: Interval,
    ) -> Result<Option<(i64, i64)>, QuantError> {
        let klines = self.load_klines(symbol, interval)?;
        if klines.is_empty() {
            return Ok(None);
        }
        let min_time = klines.first().unwrap().open_time;
        let max_time = klines.last().unwrap().open_time;
        Ok(Some((min_time, max_time)))
    }
}

/// 将 Kline 切片转为 polars DataFrame
fn klines_to_dataframe(klines: &[Kline]) -> Result<DataFrame, QuantError> {
    let open_times: Vec<i64> = klines.iter().map(|k| k.open_time).collect();
    let opens: Vec<f64> = klines.iter().map(|k| k.open).collect();
    let highs: Vec<f64> = klines.iter().map(|k| k.high).collect();
    let lows: Vec<f64> = klines.iter().map(|k| k.low).collect();
    let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
    let volumes: Vec<f64> = klines.iter().map(|k| k.volume).collect();
    let close_times: Vec<i64> = klines.iter().map(|k| k.close_time).collect();

    let df = df! {
        "open_time" => &open_times,
        "open" => &opens,
        "high" => &highs,
        "low" => &lows,
        "close" => &closes,
        "volume" => &volumes,
        "close_time" => &close_times,
    }?;

    Ok(df)
}

/// 将 polars DataFrame 转回 Kline Vec
fn dataframe_to_klines(df: &DataFrame) -> Result<Vec<Kline>, QuantError> {
    // 从 DataFrame 各列提取数据，逐行构建 Kline
    // ... 省略完整实现
    todo!()
}
```

### 增量更新策略

```
1. 调用 get_data_range() 获取本地缓存的 (min_time, max_time)
2. 计算下次请求的 start_time = max_time + interval.as_millis()
3. 调用 BinanceClient.fetch_klines(start_time = Some(start_time))
4. 将新数据与已有数据合并（save_klines 内部处理去重）
5. 如果本地无缓存，直接获取最近 N 根 K 线
```

### 设计要点

- 使用 **Parquet** 格式而非 CSV/JSON：列式存储，读写效率高，文件体积小
- 文件路径按 `{symbol}/{interval}.parquet` 组织，便于管理和查找
- `save_klines` 自动处理合并去重（按 `open_time` 去重），调用方无需关心增量逻辑
- `get_data_range` 返回缓存的时间范围，供上层决定是否需要增量获取

---

## 1.6 数据服务层

### DataService 结构体

```rust
// src/data/mod.rs

use std::sync::Arc;
use crate::common::models::{Kline, Interval};
use crate::common::error::QuantError;
use crate::data::fetcher::BinanceClient;
use crate::data::kline_store::KlineStore;

/// 数据服务层 — 整合 API 客户端和本地缓存
/// 上层模块（UI、回测）通过 DataService 获取 K 线数据
pub struct DataService {
    client: BinanceClient,
    store: KlineStore,
}

impl DataService {
    /// 创建数据服务实例
    pub fn new(client: BinanceClient, store: KlineStore) -> Self {
        Self { client, store }
    }

    /// 获取 K 线数据（优先读本地缓存，缺失时从 API 获取）
    ///
    /// # 参数
    /// - `symbol`: 交易对名称
    /// - `interval`: K 线周期
    /// - `limit`: 需要的 K 线数量
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: Interval,
        limit: u32,
    ) -> Result<Vec<Kline>, QuantError> {
        // 1. 检查本地缓存范围
        let range = self.store.get_data_range(symbol, interval)?;

        match range {
            Some((_min, max_time)) => {
                // 有缓存：增量获取最新数据
                let start_time = max_time + interval.as_millis();
                let new_klines = self.client
                    .fetch_klines(symbol, interval, Some(start_time), limit)
                    .await?;

                if !new_klines.is_empty() {
                    self.store.save_klines(symbol, interval, &new_klines)?;
                }

                // 返回最新的 limit 根 K 线
                let mut all = self.store.load_klines(symbol, interval)?;
                if all.len() > limit as usize {
                    all = all.split_off(all.len() - limit as usize);
                }
                Ok(all)
            }
            None => {
                // 无缓存：直接从 API 获取
                let klines = self.client
                    .fetch_klines(symbol, interval, None, limit)
                    .await?;
                self.store.save_klines(symbol, interval, &klines)?;
                Ok(klines)
            }
        }
    }

    /// 批量获取多个交易对的 K 线数据
    pub async fn get_klines_batch(
        &self,
        symbols: &[&str],
        interval: Interval,
        limit: u32,
    ) -> Result<Vec<(String, Vec<Kline>)>, QuantError> {
        let mut results = Vec::new();
        for symbol in symbols {
            let klines = self.get_klines(symbol, interval, limit).await?;
            results.push((symbol.to_string(), klines));
        }
        Ok(results)
    }
}
```

### 设计要点

- `DataService` 是上层模块访问数据的唯一入口，封装了缓存策略细节
- 增量更新逻辑：有缓存时只请求 `max_time` 之后的新数据，减少 API 调用
- `get_klines_batch` 串行获取多交易对数据（Phase 1 不引入并发，保持简单）
- 返回 `Vec<Kline>` 而非 DataFrame，DataFrame 转换在需要时（如回测层）进行

---

## 1.7 错误处理

### QuantError 枚举

```rust
// src/common/error.rs

use thiserror::Error;

/// 项目统一错误类型
/// 各模块的错误均可转为 QuantError 的某个变体
#[derive(Error, Debug)]
pub enum QuantError {
    /// 网络连接错误（DNS 解析失败、超时、连接重置等）
    #[error("网络错误: {0}")]
    Network(String),

    /// Binance API 返回错误（HTTP 非 2xx、频率限制等）
    #[error("API 错误: {0}")]
    Api(String),

    /// 数据解析错误（JSON 反序列化失败、字段类型不匹配等）
    #[error("解析错误: {0}")]
    Parse(String),

    /// 文件 IO 错误（缓存读写失败、目录创建失败等）
    #[error("IO 错误: {0}")]
    Io(String),

    /// polars 数据处理错误（DataFrame 操作失败、Parquet 读写异常等）
    #[error("数据处理错误: {0}")]
    DataProcess(String),
}

/// polars 错误到 QuantError 的转换
impl From<polars::error::PolarsError> for QuantError {
    fn from(err: polars::error::PolarsError) -> Self {
        QuantError::DataProcess(err.to_string())
    }
}
```

### 设计要点

- 使用 `thiserror` 派生 `Error`，自动生成 `Display` 和 `Error` trait 实现
- 每个变体携带 `String` 上下文信息，便于调试
- 为 `polars::error::PolarsError` 实现 `From`，支持 `?` 运算符自动转换
- 应用层（main.rs）使用 `anyhow` 捕获 `QuantError`，UI 层展示友好错误信息

---

## 1.8 测试策略

### 单元测试

| 测试项 | 测试内容 | 文件位置 |
|--------|---------|---------|
| Kline 解析 | Binance 原始 JSON 数组 → `Kline` 结构体转换正确 | `data/types.rs` |
| 时间戳转换 | `open_time` 毫秒 → `DateTime<Utc>` 正确性 | `common/models.rs` |
| Interval 映射 | `as_str()` / `as_millis()` 返回值正确 | `common/models.rs` |
| DataFrame 转换 | `klines_to_dataframe` / `dataframe_to_klines` 往返一致性 | `data/kline_store.rs` |

### 集成测试

| 测试项 | 测试内容 | 方法 |
|--------|---------|------|
| Mock API 响应 | 模拟 Binance `/api/v3/klines` 返回，验证完整获取流程 | 使用 `mockito` 或 `wiremock` crate 搭建本地 mock 服务器 |
| 缓存读写 | 保存 → 加载 → 验证数据一致性 | 使用 `tempfile::TempDir` 隔离测试目录 |
| 增量更新 | 首次获取 → 保存 → 增量获取 → 合并后数据完整 | mock 服务器返回不同时间段数据 |

### 验收标准

1. ✅ 能成功获取指定交易对和周期的 K 线数据并缓存到本地 Parquet 文件
2. ✅ 重复请求同一数据时直接读取本地缓存，不发起网络请求
3. ✅ 增量更新只获取缺失部分，合并后数据无重复无遗漏
4. ✅ 网络异常时返回 `QuantError::Network`，不 panic
5. ✅ 所有单元测试和集成测试通过
