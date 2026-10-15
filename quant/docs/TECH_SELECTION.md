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

| 维度 | gpui (+gpui-kit) | egui | iced | slint | tauri |
|------|------------------|------|------|-------|-------|
| **渲染模式** | GPU 渲染 + 保留模式 (Entity/Context) | 即时模式 (IMM) | 保留模式 (Elm) | 声明式 DSL | WebView |
| **语言** | 纯 Rust | 纯 Rust | 纯 Rust | Slint DSL + Rust | JS/TS + Rust |
| **文档** | gpui-kit.com 官方文档站，中英双语完整，组件逐个成篇 | 英文为主，示例分散 | 英文，中等 | 中等 | 丰富但需 JS 栈 |
| **学习曲线** | ⭐⭐ 中等（Entity/Render + Taffy Flexbox，接近前端组件化心智） | ⭐ 最低 | ⭐⭐⭐ 中等 | ⭐⭐⭐ 中等 | ⭐⭐ 需前端栈 |
| **图表生态** | 原生 CandlestickChart/LineChart/BarChart + Plot 底层绘图 | egui_plot 折线/散点，K 线需 Shape 自绘 | 需自行集成 | 无内置图表 | 依赖 JS 图表库 |
| **交易组件** | DataTable(虚拟滚动)、Dock(可停靠面板)、Input(货币掩码) | 基本控件为主 | 较少 | 较多(商业版更全) | 依赖 JS 生态 |
| **中文支持** | 系统字体直读(Windows 走 DirectWrite)，零配置 | 需手动加载中文字体文件 | 需配置字体 | 好 | 好(WebView) |
| **tokio 集成** | cx.spawn + mpsc（异步消息模型） | 原生支持 | 原生支持 | 需桥接 | 需 IPC 桥接 |
| **生产验证** | Longbridge 券商客户端（生产级交易终端） | rust-trade 等社区项目 | 无已知 | 商业嵌入式 | 大量 Web 混合应用 |

### 2.2 决策：gpui + gpui-kit

**选择 gpui（配 gpui-kit 组件库，gpui-kit 0.7.3 为 crates.io 正式版）**，理由：

1. **中英双语文档完善** — gpui-kit.com 提供完整 zh-CN 文档，chart/data-table/dock 等组件逐个成篇，比 egui 的散落示例详尽得多，对中文学习者友好
2. **原生 K 线图组件** — gpui-kit 内置 CandlestickChart（链式 API 配置 OHLC 数据源，自动渲染 OHLC tooltip），同系列另有 LineChart/BarChart/AreaChart/PieChart，另有 Plot 底层绘图库（ScaleLinear/ScaleBand）覆盖自绘场景；无需 egui 那样用 Shape API 手写 K 线绘制
3. **组件贴合交易终端** — DataTable（虚拟滚动/排序/异步 load_more 分页）、Dock（可停靠多面板/拖拽重排/布局持久化）、Input（`MaskPattern::Number` 货币掩码），正是行情终端与回测报告所需
4. **Longbridge 生产级验证** — gpui-kit 由 Longbridge 开源并在生产级券商客户端中使用，组件质量经过真实交易场景检验
5. **GPU 渲染 + 界面美观** — gpui 采用 GPU 渲染与 Taffy Flexbox 布局（Tailwind 风格链式 API），gpui-kit 提供 20+ 内置主题（明暗切换，primary/background/chart_1..5 token），观感远优于 egui 默认风格
6. **中文零配置** — 桌面端直接使用系统字体（Windows 走 DirectWrite，默认 `.SystemUIFont` 即微软雅黑），无需像 egui 那样手动加载中文字体文件

**已知约束**：

- **依赖版本陷阱**：crates.io 的 `gpui 0.2.x` 与 gpui-kit 内部封装的 `gpui-pre 0.3.9` 是两条不兼容的发布线，**严禁直接添加 `gpui = "0.2"` 依赖**（会类型冲突）；gpui-kit 重导出全部 GPUI API，`use gpui_kit::*` 即可使用
- **构建要求（Windows）**：MSVC toolchain（不能用 GNU）、CMake 在 PATH、Rust 1.92+、VS2022 Build Tools 的 C++ workload
- **保留模式心智**：状态变更后必须 `cx.notify()` 才会重渲染，比 egui 即时模式多一步心智负担，但换来 Entity/Context 组件化模型

### 2.3 淘汰方案详析

| 方案 | 淘汰原因 |
|------|----------|
| **egui** | 默认字体不含中文，需手动加载字体文件；无原生 K 线/表格/停靠面板组件（K 线需 Shape API 自绘）；文档英文为主且示例分散；即时模式全帧重绘；默认观感朴素 |
| **iced** | Elm 架构学习曲线陡峭（需理解 State/Message 单向数据流）；图表生态弱，需自行集成 K 线图 |
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

### 4.4 图表方案：gpui-kit CandlestickChart + Plot

| 方案 | 说明 | 决策 |
|------|------|------|
| **CandlestickChart** | gpui-kit 原生 K 线图组件，链式 API 配置 OHLC 数据源与阴阳线配色，自动渲染 OHLC tooltip | 用于主图 K 线展示 |
| **LineChart / BarChart / AreaChart** | gpui-kit 同系列图表组件 | 用于指标曲线（SMA/EMA/RSI/MACD）与成交量 |
| **Plot** | gpui-kit 底层绘图库（ScaleLinear / ScaleBand） | 用于自绘图（盘口深度等定制场景） |
| plotters | 通用绘图库，支持 SVG/PNG | ❌ 与 GPUI 渲染体系不兼容，集成复杂 |

**CandlestickChart 用法示例**（加密货币配色：绿涨红跌）：

```rust
CandlestickChart::new(data)          // data: Vec<Kline> 等任意结构体
    .x(|d| d.date.clone())
    .open(|d| d.open).high(|d| d.high)
    .low(|d| d.low).close(|d| d.close)
    .body_width_ratio(0.4)
    .bullish(cx.theme().success)     // 阳线色（加密货币绿涨）
    .bearish(cx.theme().danger)      // 阴线色（加密货币红跌）
// 自动渲染 OHLC tooltip，无需手绘
```

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

# GUI（gpui-kit 重导出 GPUI 核心 API：use gpui_kit::* 获得 div/Entity/Window 等；
#      组件库走 gpui_kit::component::*，布局辅助 v_flex/h_flex 走 gpui_kit::base::*。
#      严禁直接添加 gpui = "0.2" —— 与 gpui-pre 是两条不兼容发布线，同时依赖会类型冲突）
gpui-kit = "0.7.3"

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

# dev 构建性能覆盖：GUI 相关依赖以 O3 编译，避免 debug 模式下 GPU 渲染/字体排版过慢
[profile.dev.package]
gpui-pre          = { opt-level = 3 }
gpui-pre-macros   = { opt-level = 3 }
gpui-pre-platform = { opt-level = 3 }
gpui-kit          = { opt-level = 3 }
gpui-kit-assets   = { opt-level = 3 }
gpui-component    = { opt-level = 3 }
gpui-base         = { opt-level = 3 }
rustybuzz         = { opt-level = 3 }
taffy             = { opt-level = 3 }
ttf-parser        = { opt-level = 3 }
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
description = "RustQuant 桌面应用：gpui + gpui-kit GUI、回测引擎、策略框架"

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
gpui-kit = { workspace = true }   # use gpui_kit::* 得 GPUI 核心 API；组件库走 gpui_kit::component::*
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
futures-util = { workspace = true }
csv = { workspace = true }
```

**构建要求（Windows）**：MSVC toolchain（不能用 GNU）、CMake 在 PATH、Rust 1.92+、VS2022 Build Tools 的 C++ workload。gpui-kit 只进 quant-app，quant-data 保持零 GUI 依赖。

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
| **GUI** | `gpui-kit` | 0.7.x | GPUI 组件库（内含 gpui-pre 全量 API 与 CandlestickChart/LineChart/DataTable/Dock 等组件），仅进 quant-app | Phase 3 |
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
| GUI 框架 | **gpui + gpui-kit** | egui, iced, slint, tauri | 中英双语文档、原生 K 线/表格/Dock 组件、Longbridge 生产验证、GPU 渲染性能好 |
| 数据源 | **Binance API** | Tushare (A 股), Alpha Vantage (美股) | 零门槛、免费、REST+WebSocket、7×24 |
| 数据处理 | **polars** | ndarray | DataFrame 贴合 K 线、类 pandas API |
| 图表 | **gpui-kit CandlestickChart + Plot** | plotters | 原生 K 线组件、自动 OHLC tooltip、Plot 覆盖自绘场景 |
| 技术指标 | **自行实现** | ta, centaur_technical_indicators | Trait + 泛型学习价值极高 |
| 回测 | **自建向量化引擎** | barter-rs, NautilusTrader | 学习 polars + 回测原理 |
| HTTP | **reqwest** | — | Rust 最成熟 HTTP 库，原生 tokio |
| WebSocket | **tokio-tungstenite** | — | Rust WebSocket #1 crate |
| 序列化 | **serde + serde_json** | — | Rust 生态基石 |
| 错误处理 | **anyhow + thiserror** | — | 应用层 + 库层标准组合 |
| 日志 | **tracing** | env_logger | 结构化日志、异步友好 |
