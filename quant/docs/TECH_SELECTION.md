# RustQuant 技术选型方案

## 1. 选型概述

### 1.1 选型原则

| 原则 | 说明 |
|------|------|
| **学习优先** | 优先选择能补强 Rust 核心能力的库，而非"最强大"的库 |
| **前端友好** | 用户有前端背景，优先选择 API 直觉化、类前端生态的库（如 polars ≈ pandas） |
| **生态成熟** | 优先选择 crates.io 下载量高、社区活跃、文档完善的 crate |
| **零门槛验证** | 数据源选择无需注册/API Key 的方案，确保开箱即用 |

### 1.2 选型范围

本文档覆盖 RustQuant 项目所需的全部外部依赖，包括 GUI 框架、数据源、数据处理、图表、技术指标、回测引擎、HTTP/WebSocket、序列化、错误处理和日志。

---

## 2. GUI 框架选型

### 2.1 候选方案对比

| 维度 | egui | iced | slint | tauri |
|------|------|------|-------|-------|
| **渲染模式** | 即时模式 (IMM) | 保留模式 (Elm) | 声明式 DSL | WebView |
| **语言** | 纯 Rust | 纯 Rust | Slint DSL + Rust | JS/TS + Rust |
| **GitHub Stars** | ~25,000+ | ~25,000+ | ~18,000+ | ~85,000+ |
| **crates.io 下载量** | 1300 万+ | ~130 万 | ~50 万 | N/A (CLI 工具) |
| **学习曲线** | ⭐ 最低 | ⭐⭐⭐ 中等 | ⭐⭐⭐ 中等 | ⭐⭐ 需前端栈 |
| **图表生态** | egui_plot 开箱即用 | 需自行集成 | 无内置图表 | 依赖 JS 图表库 |
| **tokio 集成** | 原生支持 | 原生支持 | 需桥接 | 需 IPC 桥接 |
| **打包体积** | 4-6 MB | 10-20 MB | 5-9 MB | 50+ MB (WebView) |
| **许可证** | MIT/Apache-2.0 | MIT/Apache-2.0 | GPLv3/商业 | MIT/Apache-2.0 |
| **量化项目案例** | rust-trade 等 | 无已知 | 无已知 | 无已知 |

### 2.2 决策：egui (eframe)

**选择 egui**，理由：

1. **学习曲线最低** — 即时模式无需理解 Elm 架构或 DSL，纯 Rust 编写，前端开发者可快速上手
2. **图表生态最佳** — egui_plot 提供折线图、散点图等金融图表基础组件，Shape API 可自定义 K 线 Candlestick
3. **下载量碾压** — 1300 万+ 下载量，是 iced 的 10 倍，社区最活跃
4. **已有量化验证** — 社区已有基于 egui 的量化交易项目 (rust-trade)，证明技术可行性
5. **打包体积极小** — 4-6 MB，远小于 Tauri 的 WebView 方案

**已知局限**：即时模式每帧重绘带来功耗开销；默认字体不含中文，需手动加载字体文件。

### 2.3 淘汰方案详析

| 方案 | 淘汰原因 |
|------|----------|
| **iced** | Elm 架构学习曲线陡峭（需理解 State/Message 单向数据流）；图表生态弱，需自行集成；快速原型不如 egui 方便 |
| **slint** | 需学习 Slint DSL 额外语法；GPLv3 许可证限制闭源商用；嵌入式优化对桌面项目无意义 |
| **tauri** | 需维护前端 (JS/TS) + Rust 两套工程；WebView 打包体积 50+ MB；IPC 桥接增加复杂度；与 tokio 集成需额外处理 |

---

## 3. 数据源选型

### 3.1 市场选择分析

| 维度 | 加密货币 (Binance) | A 股 (Tushare/AKShare) | 美股 (Alpha Vantage/Yahoo) |
|------|---------------------|------------------------|---------------------------|
| **接入门槛** | 零门槛，无需注册 | 需注册获取 Token | 需 API Key，有频率限制 |
| **费用** | 完全免费 | 免费/付费分级 | 免费额度有限 |
| **REST API** | ✅ 完整 | ✅ 完整 | ✅ 完整 |
| **WebSocket 实时行情** | ✅ 免费、低延迟 | ❌ 多数需付费 | ❌ 有限制 |
| **7×24 可测试** | ✅ 全天候 | ❌ 交易时段限制 | ❌ 交易时段限制 |
| **数据粒度** | 1m/5m/15m/1h/4h/1d | 分钟级 | 分钟级 |
| **品种数量** | 200+ 交易对 | 5000+ 股票 | 10000+ 股票 |

### 3.2 决策：Binance API

**选择 Binance API（加密货币）**，理由：

1. **零门槛** — 公开行情数据无需 API Key，`GET /api/v3/klines` 即可获取 K 线数据
2. **WebSocket 免费** — `wss://stream.binance.com:9443` 提供实时推送，无频率限制
3. **全天候可测** — 加密货币 7×24 交易，随时可验证实时功能
4. **REST + WebSocket 全覆盖** — 一个数据源满足历史数据和实时数据两个学习阶段

### 3.3 Binance API 关键端点

| 端点 | 类型 | 用途 | 引入阶段 |
|------|------|------|----------|
| `GET /api/v3/klines` | REST | 获取历史 K 线数据 | Phase 1 |
| `GET /api/v3/ticker/24hr` | REST | 获取 24 小时行情摘要 | Phase 1 |
| `GET /api/v3/exchangeInfo` | REST | 获取交易对信息（精度、状态） | Phase 1 |
| `wss://stream.binance.com:9443/ws/<streamName>` | WebSocket | 实时 K 线/行情推送 | Phase 4 |

**Stream 名称格式**：
- 实时 K 线：`<symbol>@kline_<interval>`（如 `btcusdt@kline_1m`）
- 逐笔成交：`<symbol>@trade`（如 `btcusdt@trade`）
- 聚合行情：`<symbol>@ticker`

---

## 4. 核心技术栈选型

### 4.1 数据处理：polars vs ndarray

| 维度 | polars | ndarray |
|------|--------|---------|
| **抽象层级** | DataFrame（高层） | N-D 数组（底层） |
| **API 风格** | 类 pandas，链式调用 | 类 NumPy，索引操作 |
| **时间序列支持** | 原生 temporal 类型 | 需手动处理 |
| **学习曲线** | ⭐ 低（前端开发者熟悉 pandas） | ⭐⭐⭐ 高（线性代数基础） |
| **K 线数据处理** | 天然贴合 | 需自行封装 |
| **GitHub Stars** | ~37,000+ | ~5,000+ |

**决策：polars** — DataFrame 模型天然贴合 K 线时间序列，类 pandas API 对前端开发者友好，学习价值高于 ndarray 的底层数值计算。

### 4.2 HTTP 客户端：reqwest

**唯一合理选择**。reqwest 是 Rust 最成熟的 HTTP 库（下载量 4 亿+），原生 tokio 支持，API 简洁。无需对比。

### 4.3 WebSocket：tokio-tungstenite

**唯一合理选择**。tokio-tungstenite 是 Rust WebSocket 领域 #1 crate（月下载量 1900 万+），基于 tokio，提供 `Stream + Sink` 异步流模型。无需对比。

### 4.4 图表方案：egui_plot + Shape API

| 方案 | 说明 | 决策 |
|------|------|------|
| **egui_plot** | egui 生态图表库，支持折线/散点/条形图 | 用于指标曲线（SMA/EMA/RSI/MACD） |
| **Shape API** | egui 底层 Shape 绘制接口 | 用于自定义 Candlestick K 线图 |
| plotters | 通用绑图库，支持 SVG/PNG | ❌ 非即时模式，与 egui 集成复杂 |

### 4.5 技术指标：自行实现 vs 第三方库

| 方案 | 优点 | 缺点 |
|------|------|------|
| **自行实现** | 学习价值极高（Trait + 泛型 + 迭代器）、完全可控 | 开发时间较长 |
| ta (crate) | 开箱即用 | API 设计老旧，不够灵活 |
| centaur_technical_indicators | 较新 | 社区小，文档少 |

**决策：自行实现** — SMA/EMA/RSI/MACD/布林带算法不复杂，自行实现可深度练习 Trait 抽象、泛型约束和迭代器链，学习价值远超直接使用第三方库。

### 4.6 回测框架：自建 vs 第三方

| 方案 | 优点 | 缺点 |
|------|------|------|
| **自建向量化回测** | 学习 polars 数据处理、理解回测原理 | 开发量较大 |
| barter-rs | 功能完整的事件驱动回测框架 | 复杂度高，学习曲线陡峭 |
| NautilusTrader | 生产级回测引擎 | Python + Rust，过于复杂 |

**决策：自建向量化回测引擎** — 参考 barter-rs 的设计思路，用 polars 处理历史 K 线数据，实现简单的向量化回测。重点学习数据处理和文件 I/O，而非复现生产级系统。

### 4.7 序列化：serde 生态

**唯一选择**。`serde` + `serde_json` 是 Rust 生态基石（下载量 10 亿+），用于 Binance JSON 响应解析和数据持久化。

Binance K 线响应示例（需 serde 反序列化的结构）：

```rust
// Binance 返回格式：[[openTime, open, high, low, close, volume, closeTime, ...], ...]
// 使用 serde_json::Value 中间层解析嵌套数组
#[derive(Debug, Deserialize)]
pub struct RawKline {
    pub open_time: i64,
    pub open: String,    // Binance 返回字符串数字
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
}
```

### 4.8 错误处理：anyhow + thiserror

| crate | 用途 | 使用场景 | 示例 |
|-------|------|---------|------|
| `anyhow` | 应用层错误处理 | main.rs、UI 层、数据获取 | `Result<T, anyhow::Error>` |
| `thiserror` | 库层错误类型定义 | indicators、backtest、common | `#[derive(Error)] enum QuantError` |

**分工原则**：
- 库模块（indicators, backtest, data）使用 `thiserror` 定义具体错误类型，便于调用方精确匹配
- 应用层（main, ui）使用 `anyhow` 简化错误传播，用 `?` 和 `.context()` 链式附加信息

### 4.9 日志：tracing

**选择 tracing + tracing-subscriber** — 结构化日志，异步友好，支持 span 追踪，是 tokio 生态的标准日志方案。

| 对比 | tracing | env_logger |
|------|---------|------------|
| 结构化日志 | ✅ 支持 key-value 字段 | ❌ 纯文本 |
| 异步友好 | ✅ 原生支持 | ⚠️ 需额外配置 |
| span 追踪 | ✅ 可追踪请求链路 | ❌ 不支持 |
| tokio 集成 | ✅ 内置 | ❌ 无 |

---

## 5. Cargo.toml 参考配置

### 5.1 根 Cargo.toml（workspace 定义）

```toml
[workspace]
resolver = "2"
members = ["quant-data", "quant-app"]

# 统一依赖版本管理（子 crate 通过 workspace.dependencies 引用）
[workspace.dependencies]
# 异步运行时
tokio = { version = "1", features = ["full"] }

# HTTP & WebSocket
reqwest = { version = "0.12", features = ["json"] }
tokio-tungstenite = { version = "0.26", features = ["native-tls"] }
futures-util = "0.3"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 数据处理
polars = { version = "0.46", features = ["lazy", "temporal", "parquet"] }
chrono = { version = "0.4", features = ["serde"] }

# GUI
eframe = "0.31"
egui_plot = "0.32"

# 错误处理
anyhow = "1"
thiserror = "2"

# 数据导出
csv = "1.3"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 测试
tokio-test = "0.4"
```

### 5.2 quant-data/Cargo.toml（library crate）

```toml
[package]
name = "quant-data"
version = "0.1.0"
edition = "2021"
description = "RustQuant 数据层：数据获取、指标计算、实时行情（无 GUI 依赖）"

[lib]
name = "quant_data"
path = "src/lib.rs"

[dependencies]
# 继承 workspace 依赖
tokio = { workspace = true }
reqwest = { workspace = true }
tokio-tungstenite = { workspace = true }
futures-util = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
polars = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[dev-dependencies]
tokio-test = { workspace = true }
```

### 5.3 quant-app/Cargo.toml（binary crate）

```toml
[package]
name = "quant-app"
version = "0.1.0"
edition = "2021"
description = "RustQuant 桌面应用：egui GUI、回测引擎、策略框架"

[[bin]]
name = "quant-app"
path = "src/main.rs"

[dependencies]
# 项目内部 crate
quant-data = { path = "../quant-data" }

# 继承 workspace 依赖
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
polars = { workspace = true }
eframe = { workspace = true }
egui_plot = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
futures-util = { workspace = true }
csv = { workspace = true }
```

---

## 6. 完整依赖清单

| 类别 | crate | 版本 | 用途 | 引入阶段 |
|------|-------|------|------|---------|
| **异步运行时** | `tokio` | 1.x (full) | async runtime、定时器、异步 I/O | Phase 1 |
| **HTTP** | `reqwest` | 0.12.x | Binance REST API 调用 | Phase 1 |
| **序列化** | `serde` | 1.x (derive) | JSON 反序列化 | Phase 1 |
| **序列化** | `serde_json` | 1.x | JSON 编解码 | Phase 1 |
| **时间** | `chrono` | 0.4.x | 时间戳处理、K 线时间格式化 | Phase 1 |
| **数据处理** | `polars` | 0.46.x | K 线 DataFrame、回测数据处理 | Phase 1 |
| **GUI** | `eframe` | 0.31.x | egui 桌面应用框架 | Phase 3 |
| **GUI** | `egui_plot` | 0.32.x | 指标折线图、散点图 | Phase 3 |
| **WebSocket** | `tokio-tungstenite` | 0.26.x | Binance 实时行情推送 | Phase 4 |
| **异步流** | `futures-util` | 0.3.x | Stream/Sink 工具 trait | Phase 4 |
| **错误(应用)** | `anyhow` | 1.x | 应用层错误处理 | Phase 1 |
| **错误(库)** | `thiserror` | 2.x | 库层错误类型定义 | Phase 1 |
| **日志** | `tracing` | 0.1.x | 结构化日志 | Phase 1 |
| **日志** | `tracing-subscriber` | 0.3.x | 日志输出格式化 | Phase 1 |
| **测试** | `tokio-test` | 0.4.x | 异步测试工具 | Phase 2 |
| **数据导出** | `csv` | 1.3.x | CSV 数据导出 | Phase 6 |

---

## 7. 决策矩阵总览

| 决策项 | 选择 | 替代方案 | 关键理由 |
|--------|------|---------|---------|
| GUI 框架 | **egui (eframe)** | iced, slint, tauri | 学习曲线最低、egui_plot 图表生态、下载量 10x 于 iced |
| 数据源 | **Binance API** | Tushare (A 股), Alpha Vantage (美股) | 零门槛、免费、REST+WebSocket、7×24 |
| 数据处理 | **polars** | ndarray | DataFrame 贴合 K 线、类 pandas API |
| 图表 | **egui_plot + Shape** | plotters | egui 原生集成、Shape 自定义 K 线 |
| 技术指标 | **自行实现** | ta, centaur_technical_indicators | Trait + 泛型学习价值极高 |
| 回测 | **自建向量化引擎** | barter-rs, NautilusTrader | 学习 polars + 回测原理 |
| HTTP | **reqwest** | — | Rust 最成熟 HTTP 库，原生 tokio |
| WebSocket | **tokio-tungstenite** | — | Rust WebSocket #1 crate |
| 序列化 | **serde + serde_json** | — | Rust 生态基石 |
| 错误处理 | **anyhow + thiserror** | — | 应用层 + 库层标准组合 |
| 日志 | **tracing** | env_logger | 结构化日志、异步友好 |
