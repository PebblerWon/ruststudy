# RustQuant 架构设计

## 1. 架构概述

### 1.1 设计目标

| 目标 | 说明 |
|------|------|
| **模块化** | 各层独立可测，数据层/指标层/回测层均可脱离 GUI 单独测试 |
| **可测试** | 核心逻辑不依赖 UI，通过 trait 抽象支持 mock 注入 |
| **学习友好** | 分层清晰，每阶段只引入一层，渐进式构建 |

### 1.2 架构风格

采用 **分层架构 + 事件驱动** 混合模式：

- **分层架构**：上层依赖下层，下层不感知上层（单向依赖）
- **事件驱动**：实时行情通过 channel 推送，UI 通过 repaint 机制响应

---

## 2. 系统分层

```
┌─────────────────────────────────────────────────────┐
│                   策略层 (strategy)                  │
│        交易策略定义、信号生成、系统集成               │
├─────────────────────────────────────────────────────┤
│                   回测层 (backtest)                  │
│        向量化回测引擎、绩效统计、报告生成             │
├─────────────────────────────────────────────────────┤
│                   实时层 (realtime)                  │
│        WebSocket 客户端、行情推送、实时数据缓存       │
├─────────────────────────────────────────────────────┤
│                   界面层 (ui)                        │
│        egui 桌面界面、K 线图、指标图表、控制面板      │
├─────────────────────────────────────────────────────┤
│                   计算层 (indicators)                │
│        SMA/EMA/RSI/MACD/布林带、Trait 抽象          │
├─────────────────────────────────────────────────────┤
│                   数据层 (data)                      │
│        REST 数据获取、K 线存储、数据缓存             │
├─────────────────────────────────────────────────────┤
│                   公共层 (common)                    │
│        错误类型、配置、数据模型、工具函数             │
└─────────────────────────────────────────────────────┘
```

### 各层职责

| 层 | 模块名 | 职责 | 核心 crate |
|----|--------|------|-----------|
| **公共层** | `common` | 错误类型定义、全局配置、数据模型 (Kline/Ticker)、工具函数 | thiserror, chrono |
| **数据层** | `data` | Binance REST API 调用、K 线数据获取与缓存、数据格式化 | reqwest, serde, polars |
| **计算层** | `indicators` | 技术指标计算（SMA/EMA/RSI/MACD/Bollinger）、Indicator trait 抽象 | — (纯计算) |
| **界面层** | `ui` | egui 桌面界面、K 线图渲染、指标叠加、交易对选择、时间框架切换 | eframe, egui_plot |
| **实时层** | `realtime` | Binance WebSocket 连接管理、实时行情解析、mpsc 推送至 UI | tokio-tungstenite, futures-util |
| **回测层** | `backtest` | 向量化回测引擎、历史数据加载、收益/回撤/夏普比率计算 | polars |
| **策略层** | `strategy` | Strategy trait 定义、内置策略（均线交叉、RSI 超买超卖）、信号生成 | — |

---

## 3. 项目结构

```
quant/
├── docs/
│   ├── PRD.md                    # 产品需求文档
│   ├── DEV_PLAN.md               # 开发计划
│   ├── TECH_SELECTION.md         # 技术选型方案
│   └── ARCHITECTURE.md           # 架构设计（本文件）
├── src/
│   ├── common/
│   │   ├── mod.rs                # 公共模块入口
│   │   ├── error.rs              # 统一错误类型 (QuantError)
│   │   ├── config.rs             # 全局配置（API 地址、刷新间隔等）
│   │   └── models.rs             # 核心数据模型 (Kline, Ticker, Trade)
│   ├── data/
│   │   ├── mod.rs                # 数据层入口
│   │   ├── fetcher.rs            # Binance REST API 数据获取
│   │   ├── kline_store.rs        # K 线数据缓存与管理
│   │   └── types.rs              # Binance API 响应类型定义
│   ├── indicators/
│   │   ├── mod.rs                # 指标层入口、Indicator trait 定义
│   │   ├── sma.rs                # 简单移动平均线
│   │   ├── ema.rs                # 指数移动平均线
│   │   ├── rsi.rs                # 相对强弱指标
│   │   ├── macd.rs               # MACD 指标
│   │   └── bollinger.rs          # 布林带
│   ├── ui/
│   │   ├── mod.rs                # UI 层入口
│   │   ├── app.rs                # 主应用状态与布局
│   │   ├── chart.rs              # K 线图 (Candlestick via Shape API)
│   │   ├── indicator_panel.rs    # 指标叠加面板
│   │   ├── control_panel.rs      # 交易对/时间框架选择控件
│   │   └── theme.rs              # 颜色主题与样式配置
│   ├── realtime/
│   │   ├── mod.rs                # 实时层入口
│   │   ├── ws_client.rs          # WebSocket 客户端（连接管理、重连）
│   │   └── handler.rs            # 消息解析与分发
│   ├── backtest/
│   │   ├── mod.rs                # 回测层入口
│   │   ├── engine.rs             # 向量化回测引擎
│   │   ├── report.rs             # 绩效报告（收益/回撤/夏普比率）
│   │   └── data_loader.rs        # 历史数据加载（从 KlineStore）
│   ├── strategy/
│   │   ├── mod.rs                # 策略层入口、Strategy trait 定义
│   │   ├── ma_cross.rs           # 均线交叉策略
│   │   └── rsi_reversal.rs       # RSI 超买超卖策略
│   ├── main.rs                   # 程序入口、启动 eframe
│   └── lib.rs                    # 库入口、模块声明
├── Cargo.toml
└── tests/
    ├── data_tests.rs             # 数据层集成测试
    ├── indicator_tests.rs        # 指标计算单元测试
    └── backtest_tests.rs         # 回测引擎测试
```

### 文件职责表

| 文件 | 职责 | 所属层 | 引入阶段 |
|------|------|--------|---------|
| `common/error.rs` | `QuantError` 枚举，统一错误类型 | common | Phase 1 |
| `common/config.rs` | API 地址、刷新间隔、默认交易对 | common | Phase 1 |
| `common/models.rs` | `Kline`, `Ticker`, `Interval` 等核心模型 | common | Phase 1 |
| `data/fetcher.rs` | `DataFetcher` — 调用 Binance REST 获取 K 线 | data | Phase 1 |
| `data/kline_store.rs` | `KlineStore` — 内存 K 线缓存、polars DataFrame | data | Phase 1 |
| `data/types.rs` | Binance API JSON 响应的 serde 反序列化类型 | data | Phase 1 |
| `indicators/mod.rs` | `Indicator` trait 定义、指标注册 | indicators | Phase 2 |
| `indicators/sma.rs` ~ `bollinger.rs` | 各指标实现 | indicators | Phase 2 |
| `ui/app.rs` | `QuantApp` 主状态、`eframe::App` 实现 | ui | Phase 3 |
| `ui/chart.rs` | K 线 Candlestick 渲染 (Shape API) | ui | Phase 3 |
| `ui/indicator_panel.rs` | 指标曲线叠加 (egui_plot) | ui | Phase 3 |
| `ui/control_panel.rs` | 交易对选择、时间框架、指标开关 | ui | Phase 3 |
| `realtime/ws_client.rs` | WebSocket 连接、自动重连、消息流 | realtime | Phase 4 |
| `realtime/handler.rs` | 解析 Binance WS 消息、转换为 Ticker | realtime | Phase 4 |
| `backtest/engine.rs` | 向量化回测核心逻辑 | backtest | Phase 5 |
| `backtest/report.rs` | 回测绩效统计与报告输出 | backtest | Phase 5 |
| `backtest/data_loader.rs` | 从 KlineStore 加载历史数据到 polars | backtest | Phase 5 |
| `strategy/mod.rs` | `Strategy` trait 定义 | strategy | Phase 6 |
| `strategy/ma_cross.rs` | 均线交叉策略实现 | strategy | Phase 6 |
| `strategy/rsi_reversal.rs` | RSI 超买超卖策略实现 | strategy | Phase 6 |

---

## 4. 核心数据流

### 4.1 历史数据流

```
Binance REST API
      │
      ▼
┌──────────────┐
│  DataFetcher │  reqwest GET /api/v3/klines
└──────┬───────┘
       │ Vec<Kline>
       ▼
┌──────────────┐
│  KlineStore  │  内存缓存 + polars DataFrame
└──────┬───────┘
       │ DataFrame
       ▼
┌──────────────┐
│  Indicator   │  SMA/EMA/RSI/MACD 计算
└──────┬───────┘
       │ Vec<f64>
       ▼
┌──────────────┐
│  UI (egui)   │  K 线图 + 指标曲线渲染
└──────────────┘
```

### 4.2 实时数据流

```
Binance WebSocket (wss://stream.binance.com:9443)
      │
      ▼
┌──────────────┐
│  WsClient    │  tokio-tungstenite 连接管理
│  (tokio task)│
└──────┬───────┘
       │ mpsc::Sender<Ticker>
       ▼
┌──────────────┐
│  UI (egui)   │  ctx.request_repaint() 触发重绘
│  接收 rx     │  读取最新 Ticker 更新状态
└──────────────┘
```

### 4.3 回测数据流

```
┌──────────────┐
│  KlineStore  │  历史 K 线数据
└──────┬───────┘
       │ polars DataFrame
       ▼
┌──────────────┐
│  BacktestEngine │  向量化回测：逐 bar 计算信号
└──────┬───────┘
       │ Signal (Buy/Sell/Hold)
       ▼
┌──────────────┐
│  Strategy    │  策略逻辑：基于 Indicator 输出决策
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Report      │  总收益、最大回撤、夏普比率
└──────────────┘
```

---

## 5. 异步模型

### 5.1 tokio runtime 配置

```rust
#[tokio::main]
async fn main() -> eframe::Result {
    // 初始化 tracing
    tracing_subscriber::fmt::init();

    // eframe 启动（内部运行 tokio runtime）
    let options = eframe::NativeOptions::default();
    eframe::run_native("RustQuant", options, Box::new(|cc| Ok(Box::new(QuantApp::new(cc)))))
}
```

### 5.2 任务拓扑

| tokio task | 职责 | 通信方式 |
|------------|------|---------|
| **主线程** | egui UI 渲染、用户交互 | — |
| **ws_receiver** | 接收 WebSocket 消息，解析为 `Ticker` | `mpsc::Sender<Ticker>` → UI |
| **data_fetch** (按需) | 异步加载历史 K 线数据 | `oneshot::Sender<Vec<Kline>>` → UI |

### 5.3 与 egui 的集成模式

egui 是即时模式 GUI，运行在主线程。异步任务通过 channel 将数据传递给 UI：

```rust
// UI 侧：持有 receiver
struct QuantApp {
    rx: mpsc::UnboundedReceiver<Ticker>,
    latest_ticker: Option<Ticker>,
    ctx: egui::Context,  // 用于 request_repaint
}

// 在 update() 中消费消息
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // 非阻塞读取所有待处理消息
    while let Ok(ticker) = self.rx.try_recv() {
        self.latest_ticker = Some(ticker);
    }
    // 渲染 K 线图...
}
```

### 5.4 优雅关闭策略

- WebSocket 客户端持有 `CancellationToken`（tokio-util），UI 关闭时触发 cancel
- `ws_receiver` task 检测到 cancel 后主动关闭连接并退出
- 主线程等待所有 task join 后退出

---

## 6. 模块间通信

### 6.1 依赖关系

```
strategy ──→ indicators ──→ common
    │              │
    ▼              ▼
backtest ──→ data ──→ common
    │           │
    ▼           ▼
   ui ←── realtime ──→ common
```

### 6.2 关键 trait 定义

```rust
// indicators/mod.rs — 指标抽象
pub trait Indicator {
    /// 指标名称
    fn name(&self) -> &str;
    /// 计算指标值（输入收盘价序列，输出指标值序列）
    fn compute(&self, closes: &[f64]) -> Vec<f64>;
}

// strategy/mod.rs — 策略抽象
pub enum Signal { Buy, Sell, Hold }

pub trait Strategy {
    /// 策略名称
    fn name(&self) -> &str;
    /// 根据当前指标值生成交易信号
    fn generate_signal(&self, indicators: &IndicatorValues) -> Signal;
}

// data/mod.rs — 数据源抽象（便于 mock 测试）
#[async_trait]
pub trait DataProvider: Send + Sync {
    /// 获取 K 线数据
    async fn fetch_klines(
        &self,
        symbol: &str,
        interval: Interval,
        limit: u32,
    ) -> Result<Vec<Kline>>;
}
```

### 6.3 错误传播策略

| 层 | 错误类型 | 策略 |
|----|---------|------|
| common | `QuantError` (thiserror) | 定义统一错误枚举 |
| data | `DataError` → `QuantError` | 网络/解析错误转为 `QuantError::Data` |
| indicators | 不返回错误 | 纯计算函数，输入无效时返回空 Vec |
| realtime | `WsError` → `QuantError` | 连接/解析错误转为 `QuantError::WebSocket` |
| ui | `anyhow::Result` | 顶层捕获，展示错误对话框 |

---

## 7. 数据模型概览

### 7.1 核心数据结构

```rust
/// K 线数据（common/models.rs）
pub struct Kline {
    pub open_time: i64,      // 开盘时间 (ms timestamp)
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: i64,     // 收盘时间
}

/// 实时行情（common/models.rs）
pub struct Ticker {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: i64,
}

/// 时间间隔枚举
pub enum Interval {
    M1, M5, M15, M30, H1, H4, D1,
}

/// 指标值集合（传递给策略层）
pub struct IndicatorValues {
    pub closes: Vec<f64>,
    pub sma: Option<Vec<f64>>,
    pub ema: Option<Vec<f64>>,
    pub rsi: Option<Vec<f64>>,
    pub macd: Option<MacdResult>,
    pub bollinger: Option<BollingerResult>,
}
```

### 7.2 数据模型关系

```
Kline ──→ Indicator ──→ IndicatorValues ──→ Strategy ──→ Signal
  │                                                  ▲
  └──→ KlineStore ──→ BacktestEngine ──→ Report ────┘
         │
Ticker ──┘ (实时更新)
```

---

## 8. 阶段演进

| 阶段 | 新增模块 | 新增依赖 | 累计模块数 |
|------|---------|---------|-----------|
| **Phase 1** | common, data | tokio, reqwest, serde, chrono, polars, anyhow, tracing | 2 |
| **Phase 2** | indicators | thiserror | 3 |
| **Phase 3** | ui | eframe, egui_plot | 4 |
| **Phase 4** | realtime | tokio-tungstenite, futures-util | 5 |
| **Phase 5** | backtest | — (复用 polars) | 6 |
| **Phase 6** | strategy | — (无新依赖) | 7 |

每个阶段只引入一个新模块，确保学习者能集中精力理解该层的 Rust 特性重点：

- **Phase 1** — async/await + reqwest + serde 实战
- **Phase 2** — Trait 抽象 + 泛型 + 迭代器链
- **Phase 3** — egui 即时模式 GUI + 状态管理
- **Phase 4** — WebSocket + mpsc channel + 并发 task
- **Phase 5** — polars 数据处理 + 文件 I/O + 统计计算
- **Phase 6** — 设计模式 (Strategy) + 完整系统集成
