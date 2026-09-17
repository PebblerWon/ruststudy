# RustQuant 开发计划

## 总体安排

- **总工期：** 约 8-10 周（每周 10-15 小时）
- **6 个阶段：** 数据层 → 指标层 → 界面层 → 实时层 → 回测层 → 策略层
- **学习导向：** 每阶段引入一层新模块，聚焦该阶段 Rust 特性重点

### Crate 结构

| Crate | 类型 | 包含模块 | 说明 |
|-------|------|---------|------|
| **quant-data** | library | data + indicators + realtime | 无 GUI 依赖，可独立测试 |
| **quant-app** | binary | ui + backtest + strategy | 依赖 quant-data，含 main.rs |

| 里程碑 | 阶段 | 核心交付物 | 预估 |
|--------|------|-----------|------|
| M1 | Phase 1-2 | 数据获取 + 指标计算（quant-data，CLI 可验证） | ~3 周 |
| M2 | Phase 3 | egui 桌面应用 + K 线图（quant-app） | ~2 周 |
| M3 | Phase 4 | WebSocket 实时行情 + UI 刷新（quant-data + quant-app 集成） | ~1.5 周 |
| M4 | Phase 5-6 | 回测引擎 + 策略框架 + 完整系统（quant-app） | ~3 周 |

---

## Phase 1 — 数据层（~10 天）

> 目标：接入 Binance REST API，获取历史 K 线并本地持久化
> 补强：async/await 实战、reqwest、serde 序列化

- [ ] **T1.1 项目初始化** — cargo workspace 创建 `quant-data`(lib) + `quant-app`(bin) 两个 crate，配置各自 Cargo.toml 依赖
  - **产出：** 可编译的 workspace 骨架（quant-data 空 lib.rs + quant-app 空 main.rs） | **学习点：** cargo workspace、多 crate 依赖管理
  - **技术方案：** 见 [tech/01_data_layer.md § 1.2](tech/01_data_layer.md)

- [ ] **T1.2 公共层：错误类型与数据模型** — QuantError(thiserror)、Kline/Interval、Binance API 响应类型(serde Deserialize)，均放入 **quant-data**
  - **产出：** quant-data/src/models.rs + error.rs + data/types.rs | **学习点：** serde derive、`#[serde(rename)]`、thiserror
  - **技术方案：** 见 [tech/01_data_layer.md § 1.3](tech/01_data_layer.md)

- [ ] **T1.3 Binance REST 客户端** — DataFetcher 实现 DataProvider trait(async_trait)，reqwest GET /api/v3/klines，放入 **quant-data**
  - **产出：** quant-data/src/data/fetcher.rs | **学习点：** reqwest 异步 HTTP、async_trait
  - **技术方案：** 见 [tech/01_data_layer.md § 1.4](tech/01_data_layer.md)

- [ ] **T1.4 数据持久化** — KlineStore Parquet 文件存储（`{symbol}/{interval}.parquet`），放入 **quant-data**
  - **产出：** quant-data/src/data/kline_store.rs | **学习点：** 文件 I/O、polars Parquet 读写、缓存策略
  - **技术方案：** 见 [tech/01_data_layer.md § 1.5](tech/01_data_layer.md)

- [ ] **T1.5 数据层测试** — 在 **quant-data** 内编写单元测试 + tests/data_tests.rs 集成测试，mock HTTP 验证解析、缓存命中/未命中
  - **产出：** quant-data/tests/data_tests.rs | **学习点：** async 测试、mock 注入、tempfile 隔离
  - **技术方案：** 见 [tech/01_data_layer.md § 1.6](tech/01_data_layer.md)

**阶段验收：** `cargo run -p quant-app -- fetch --symbol BTCUSDT --interval 1h --limit 500` 打印 K 线表格，重复执行走缓存

---

## Phase 2 — 指标层（~8 天）

> 目标：自行实现 5 种技术指标，建立 Indicator trait 抽象
> 补强：Trait 抽象、泛型、迭代器链、数值计算

- [ ] **T2.1 Indicator trait 设计** — `Indicator` trait（name + compute）、IndicatorValues 结构体，放入 **quant-data**
  - **产出：** quant-data/src/indicators/mod.rs | **学习点：** Trait 抽象、动态分发 vs 静态分发
  - **技术方案：** 见 [tech/02_indicator_layer.md § 2.2](tech/02_indicator_layer.md)

- [ ] **T2.2 SMA 与 EMA** — 滑动窗口(SMA) + 递推公式(EMA)
  - **产出：** quant-data/src/indicators/sma.rs + ema.rs | **学习点：** 迭代器链(window/map)、f64 精度
  - **技术方案：** 见 [tech/02_indicator_layer.md § 2.3](tech/02_indicator_layer.md)

- [ ] **T2.3 RSI 与 MACD** — RS 比值平滑(RSI) + DIF/DEA/柱状图(MACD, IndicatorOutput::Multi)
  - **产出：** quant-data/src/indicators/rsi.rs + macd.rs | **学习点：** 复杂数值算法、IndicatorOutput::Multi 多值输出
  - **技术方案：** 见 [tech/02_indicator_layer.md § 2.4-2.5](tech/02_indicator_layer.md)

- [ ] **T2.4 布林带** — 中轨(SMA) + 上轨(+2σ) + 下轨(-2σ)
  - **产出：** quant-data/src/indicators/bollinger.rs | **学习点：** 标准差计算、浮点精度
  - **技术方案：** 见 [tech/02_indicator_layer.md § 2.6](tech/02_indicator_layer.md)

- [ ] **T2.5 指标管道与测试** — IndicatorPipeline 管道 + TradingView 交叉验证（误差 < 0.01%），测试在 **quant-data** 内
  - **产出：** quant-data/tests/indicator_tests.rs | **学习点：** 管道模式、浮点近似断言
  - **技术方案：** 见 [tech/02_indicator_layer.md § 2.7-2.8](tech/02_indicator_layer.md)

**阶段验收：** `Sma::new(20).compute(&closes)` 输出正确长度，值与 TradingView 误差 < 0.01%

---

## Phase 3 — 界面层（~12 天）

> 目标：构建 egui 桌面应用，K 线图 + 指标面板 + 品种切换（quant-app，依赖 quant-data）
> 补强：egui 即时模式 GUI、布局、状态管理、自绘图元

- [ ] **T3.1 eframe 应用入口** — QuantApp 实现 eframe::App，主窗口四区布局，quant-app 依赖 quant-data
  - **产出：** quant-app/src/ui/app.rs + theme.rs | **学习点：** eframe 生命周期、SidePanel/CentralPanel
  - **技术方案：** 见 [tech/03_ui_layer.md § 3.1](tech/03_ui_layer.md)

- [ ] **T3.2 品种列表与控件** — 左侧交易对列表 + 顶部时间框架切换
  - **产出：** quant-app/src/ui/control_panel.rs | **学习点：** egui 控件、状态绑定
  - **技术方案：** 见 [tech/03_ui_layer.md § 3.2](tech/03_ui_layer.md)

- [ ] **T3.3 K 线图渲染** — Shape API 自绘 Candlestick（红绿蜡烛），缩放 + 平移
  - **产出：** quant-app/src/ui/chart.rs | **学习点：** Shape API、坐标变换、自绘图元
  - **技术方案：** 见 [tech/03_ui_layer.md § 3.3](tech/03_ui_layer.md)

- [ ] **T3.4 指标叠加与成交量** — K 线图叠加 SMA/EMA 曲线 + 成交量子图
  - **产出：** quant-app/src/ui/indicator_panel.rs | **学习点：** 曲线绘制、多子图布局
  - **技术方案：** 见 [tech/03_ui_layer.md § 3.4](tech/03_ui_layer.md)

- [ ] **T3.5 数据接入与状态管理** — 品种/周期切换触发 quant-data 加载、loading 态、app 状态流转
  - **产出：** quant-app 完整状态管理 | **学习点：** GUI 状态管理、跨 crate 异步数据加载与 UI 响应
  - **技术方案：** 见 [tech/03_ui_layer.md § 3.5](tech/03_ui_layer.md)

**阶段验收：** `cargo run -p quant-app` 启动窗口，切换品种/周期，K 线图渲染 + 指标曲线叠加

---

## Phase 4 — 实时层（~8 天）

> 目标：接入 Binance WebSocket，实时行情推送与 UI 刷新（quant-data 负责 WS，quant-app 消费展示）
> 补强：tokio-tungstenite、mpsc channel、并发 task、断线重连

- [ ] **T4.1 WebSocket 客户端** — WsClient 连接 Binance WS，订阅多交易对 K 线 + Ticker，放入 **quant-data**
  - **产出：** quant-data/src/realtime/ws_client.rs | **学习点：** tokio-tungstenite、异步流
  - **技术方案：** 见 [tech/04_realtime_layer.md § 4.2](tech/04_realtime_layer.md)

- [ ] **T4.2 消息解析与分发** — Ticker/WsKline 模型、Binance WS JSON 解析，放入 **quant-data**
  - **产出：** quant-data/src/realtime/handler.rs | **学习点：** 嵌套 JSON 反序列化、消息分发
  - **技术方案：** 见 [tech/04_realtime_layer.md § 4.3](tech/04_realtime_layer.md)

- [ ] **T4.3 通道架构与后台任务** — mpsc::UnboundedSender 推送 + CancellationToken 优雅关闭，放入 **quant-data**
  - **产出：** quant-data channel 通信 + task 管理 | **学习点：** mpsc channel、CancellationToken
  - **技术方案：** 见 [tech/04_realtime_layer.md § 4.4-4.5](tech/04_realtime_layer.md)

- [ ] **T4.4 UI 实时集成** — quant-app 通过 try_recv 消费 quant-data 推送、价格闪烁(涨绿跌红)、K 线自动追加、状态栏
  - **产出：** quant-app 实时 UI | **学习点：** egui request_repaint、跨线程/跨 crate GUI 更新
  - **技术方案：** 见 [tech/04_realtime_layer.md § 4.6](tech/04_realtime_layer.md)

- [ ] **T4.5 断线重连** — 指数退避(1s→2s→4s→30s)、状态栏连接状态
  - **产出：** quant-data 稳定 WS 连接 | **学习点：** 错误恢复、重试模式
  - **技术方案：** 见 [tech/04_realtime_layer.md § 4.8](tech/04_realtime_layer.md)

**阶段验收：** 实时价格闪烁，K 线自动追加，断网恢复后自动重连

---

## Phase 5 — 回测层（~10 天）

> 目标：向量化回测引擎 + 绩效报告（quant-app，读取 quant-data 数据）
> 补强：polars 数据处理、统计计算

- [ ] **T5.1 回测引擎核心** — BacktestEngine 逐 bar 信号 + 手续费/滑点模拟，放入 **quant-app**
  - **产出：** quant-app/src/backtest/engine.rs | **学习点：** 向量化计算、金融精度
  - **技术方案：** 见 [tech/05_backtest_layer.md § 5.1](tech/05_backtest_layer.md)

- [ ] **T5.2 数据加载器** — DataLoader 从 quant-data KlineStore 加载到 polars DataFrame，放入 **quant-app**
  - **产出：** quant-app/src/backtest/data_loader.rs | **学习点：** polars DataFrame 操作
  - **技术方案：** 见 [tech/05_backtest_layer.md § 5.2](tech/05_backtest_layer.md)

- [ ] **T5.3 绩效统计** — 总收益/年化/夏普/最大回撤/胜率/盈亏比
  - **产出：** quant-app/src/backtest/report.rs | **学习点：** 统计计算公式
  - **技术方案：** 见 [tech/05_backtest_layer.md § 5.3](tech/05_backtest_layer.md)

- [ ] **T5.4 回测报告 UI** — 收益曲线图 + 统计表格 + 交易记录 + 参数配置面板，放入 **quant-app**
  - **产出：** quant-app 回测报告界面 | **学习点：** egui 图表、表格布局
  - **技术方案：** 见 [tech/05_backtest_layer.md § 5.4](tech/05_backtest_layer.md)

- [ ] **T5.5 回测测试** — 在 **quant-app** 内编写测试，简单策略验证统计正确性 + 手续费/滑点影响
  - **产出：** quant-app/tests/backtest_tests.rs | **学习点：** 金融计算测试
  - **技术方案：** 见 [tech/05_backtest_layer.md § 5.5](tech/05_backtest_layer.md)

**阶段验收：** UI 选择 BTCUSDT/1h，配置均线交叉策略，回测展示收益曲线 + 夏普比率 + 交易记录

---

## Phase 6 — 策略层（~8 天）

> 目标：策略框架 + 内置策略 + 模拟交易 + 数据导出（quant-app，完整系统集成）
> 补强：设计模式(Strategy)、模块化、完整系统集成

- [ ] **T6.1 Strategy trait 框架** — Strategy trait(name + default_params + init + on_kline + reset)、Signal 枚举、`Box<dyn Strategy>` 动态分发，放入 **quant-app**
  - **产出：** quant-app/src/strategy/mod.rs | **学习点：** Strategy 设计模式、trait object
  - **技术方案：** 见 [tech/06_strategy_layer.md § 6.2](tech/06_strategy_layer.md)

- [ ] **T6.2 均线交叉策略** — 金叉 Buy / 死叉 Sell，参数可配(短/长周期)
  - **产出：** quant-app/src/strategy/sma_cross.rs | **学习点：** 交叉检测算法、状态跟踪
  - **技术方案：** 见 [tech/06_strategy_layer.md § 6.3](tech/06_strategy_layer.md)

- [ ] **T6.3 RSI 超买超卖策略** — RSI<30 Buy / RSI>70 Sell，参数可配
  - **产出：** quant-app/src/strategy/rsi_strategy.rs | **学习点：** 阈值策略、参数化设计
  - **技术方案：** 见 [tech/06_strategy_layer.md § 6.4](tech/06_strategy_layer.md)

- [ ] **T6.4 模拟交易** — Paper Trading：持仓跟踪 + 账户余额 + 盈亏计算 + 手动下单/平仓，放入 **quant-app**
  - **产出：** quant-app 模拟交易模块 | **学习点：** 状态机、金融计算
  - **技术方案：** 见 [tech/06_strategy_layer.md § 6.5](tech/06_strategy_layer.md)

- [ ] **T6.5 数据导出与系统整合** — CSV 导出(BOM) + quant-data ↔ quant-app 全系统集成测试
  - **产出：** 完整桌面量化系统（quant-app + quant-data）| **学习点：** CSV 序列化、跨 crate 系统集成测试
  - **技术方案：** 见 [tech/06_strategy_layer.md § 6.7-6.8](tech/06_strategy_layer.md)

**阶段验收：** 实时 K 线 → 策略回测 → 参数调优 → CSV 导出，全流程可用

---

## 进度追踪表

| 任务 | 状态 | 学习点 | 备注 |
|------|------|--------|------|
| T1.1 | ⬜ | cargo workspace、多 crate 依赖 | quant-data(lib) + quant-app(bin) 骨架 |
| T1.2 | ⬜ | serde derive、thiserror | 数据模型 + 错误类型（quant-data） |
| T1.3 | ⬜ | reqwest、async_trait | REST 客户端（quant-data） |
| T1.4 | ⬜ | 文件 I/O、缓存策略 | Parquet 持久化（quant-data） |
| T1.5 | ⬜ | async 测试、mock 注入 | 数据层测试（quant-data） |
| T2.1 | ⬜ | Trait 抽象、动态分发 | Indicator trait（quant-data） |
| T2.2 | ⬜ | 迭代器链、f64 精度 | SMA + EMA（quant-data） |
| T2.3 | ⬜ | 数值算法、Multi 输出 | RSI + MACD（quant-data） |
| T2.4 | ⬜ | 标准差、浮点精度 | 布林带（quant-data） |
| T2.5 | ⬜ | 管道模式、近似断言 | 指标管道 + 测试（quant-data） |
| T3.1 | ⬜ | eframe、即时模式 GUI | 应用入口 + 布局（quant-app） |
| T3.2 | ⬜ | egui 控件、状态绑定 | 品种列表 + 控件（quant-app） |
| T3.3 | ⬜ | Shape API、坐标变换 | K 线图自绘（quant-app） |
| T3.4 | ⬜ | 曲线绘制、多子图 | 指标叠加 + 成交量（quant-app） |
| T3.5 | ⬜ | GUI 状态管理 | 数据接入 + 状态流转（quant-app） |
| T4.1 | ⬜ | tokio-tungstenite、异步流 | WebSocket 客户端（quant-data） |
| T4.2 | ⬜ | 嵌套 JSON 解析 | 消息处理（quant-data） |
| T4.3 | ⬜ | mpsc channel、CancellationToken | 通道架构（quant-data） |
| T4.4 | ⬜ | request_repaint、跨线程 UI | 实时 UI 集成（quant-app 消费 quant-data 数据） |
| T4.5 | ⬜ | 指数退避、错误恢复 | 断线重连（quant-data） |
| T5.1 | ⬜ | 向量化计算、金融精度 | 回测引擎（quant-app） |
| T5.2 | ⬜ | polars DataFrame | 数据加载（quant-app，读 quant-data） |
| T5.3 | ⬜ | 统计计算 | 绩效统计（quant-app） |
| T5.4 | ⬜ | egui 图表、表格布局 | 回测报告 UI（quant-app） |
| T5.5 | ⬜ | 金融计算测试 | 回测测试（quant-app） |
| T6.1 | ⬜ | Strategy 模式、trait object | 策略框架（quant-app） |
| T6.2 | ⬜ | 交叉检测、状态跟踪 | 均线交叉策略（quant-app） |
| T6.3 | ⬜ | 阈值策略、参数化 | RSI 策略（quant-app） |
| T6.4 | ⬜ | 状态机、金融计算 | 模拟交易（quant-app） |
| T6.5 | ⬜ | CSV 序列化、系统集成 | 数据导出 + 整合（quant-app） |

**状态说明：** ⬜ 待开始 | 🔵 进行中 | ✅ 已完成 | ⏸ 暂停 | ❌ 已取消
