# Stock Dashboard 开发计划

## 总体安排

- **总工期：** MVP 约 3 周（每周 10-15 小时），Phase 4 可选扩展约 2 周
- **4 个阶段：** 数据层 → 界面层 → 轮询刷新 → 扩展页面（可选）
- **学习导向：** 先用 CLI 打通真实数据源，再上 GUI；每阶段一个聚焦主题
- **参照：** 架构分层与任务格式参照 quant 项目；产品形态对标 [chengzuopeng/stock-dashboard](https://github.com/chengzuopeng/stock-dashboard)

### Crate 结构

| Crate | 类型 | 包含模块 | 说明 |
|-------|------|---------|------|
| **stock-data** | library | common + data | 东财客户端、模型、缓存，无 GUI 依赖 |
| **stock-app** | binary | ui | gpui 桌面应用，依赖 stock-data |

| 里程碑 | 阶段 | 核心交付物 | 预估 |
|--------|------|-----------|------|
| M1 | Phase 1 | 东财数据层（CLI 可验证） | ~1 周 |
| M2 | Phase 2 | 总览 + 自选 + K 线桌面应用 | ~1.5 周 |
| M3 | Phase 3 | 轮询实时刷新完整 MVP | ~0.5 周 |
| M4 | Phase 4 | 分时/热力图/榜单（可选） | ~2 周 |

---

## Phase 1 — 数据层（~5 天）

> 目标：接入东方财富公开接口，提供快照/指数/K 线/分时四类数据（stock-data，CLI 可验证）
> 补强：reqwest 请求头与限速、serde 容错反序列化、纯函数设计、TTL 缓存

- [ ] **T1.1 项目初始化** — cargo workspace 创建 `stock-data`(lib) + `stock-app`(bin)，配置依赖（tokio/reqwest/serde/thiserror/chrono）
  - **产出：** 可编译 workspace 骨架 | **学习点：** cargo workspace、多 crate 依赖管理
  - **技术方案：** 见 [tech/01_data_layer.md § 1.2](tech/01_data_layer.md)
- [ ] **T1.2 公共层：错误与模型** — DataError(thiserror)、Quote/IndexQuote/TimelinePoint/Kline/MarketSnapshot 模型、`"-"` 停牌容错反序列化，放入 **stock-data**
  - **产出：** stock-data/src/common/ | **学习点：** thiserror、自定义 Deserialize
  - **技术方案：** 见 [tech/01_data_layer.md § 1.3](tech/01_data_layer.md)
- [ ] **T1.3 东财客户端骨架** — EastmoneyClient（UA + Referer 默认头、令牌桶限速 ≥500ms）、secid 市场规则纯函数（沪 1./深 0.，拒绝北交所），放入 **stock-data**
  - **产出：** stock-data/src/data/eastmoney.rs | **学习点：** reqwest Client 默认头、令牌桶、纯函数单测
  - **技术方案：** 见 [tech/01_data_layer.md § 1.4](tech/01_data_layer.md)
- [ ] **T1.4 数据解析与归一** — ulist 批量快照 + clist 全市场分页 + kline CSV 解析 + trends2 分时；normalize 单位归一（元→万/亿、手→股），放入 **stock-data**
  - **产出：** eastmoney_types.rs + normalize.rs | **学习点：** 分页拼装、CSV 手工解析、单位契约
  - **技术方案：** 见 [tech/01_data_layer.md § 1.5](tech/01_data_layer.md)
- [ ] **T1.5 缓存与测试** — 内存 TTL 缓存（行情 5s/K 线 10min/快照 60s）；fixture mock 测试 + `#[ignore]` 真实接口 smoke test，放入 **stock-data**
  - **产出：** cache.rs + tests/ | **学习点：** 缓存设计、fixture 管理、tempfile 隔离
  - **技术方案：** 见 [tech/01_data_layer.md § 1.6-1.7](tech/01_data_layer.md)

**阶段验收：** `cargo run -p stock-app -- fetch --code 600519` 打印茅台日线表格（OHLC 有序、前复权正确）；`--indexes` 打印 8 大指数快照

---

## Phase 2 — 界面层（~7 天）

> 目标：gpui 桌面应用，总览 + 自选 + K 线三大面板（stock-app，依赖 stock-data）
> 补强：gpui Entity/Render 模型、v_flex/h_flex 布局、gpui-kit 组件、语义主题色

- [ ] **T2.1 应用骨架与主题** — application() 三段式启动、主窗口布局（单页 or Dock）、theme.rs 定义 A 股语义色（up_color 红 / down_color 绿），放入 **stock-app**
  - **产出：** stock-app/src/ui/app.rs + theme.rs | **学习点：** gpui 生命周期、主题设计
  - **技术方案：** 见 [tech/02_ui_layer.md § 2.2](tech/02_ui_layer.md)
- [ ] **T2.2 总览面板** — 8 大指数卡片 grid（ulist 拉取）+ 市场涨跌统计 + 涨跌停 + 总成交额（clist 全量统计），放入 **stock-app**
  - **产出：** ui/overview.rs | **学习点：** gpui 卡片布局、Entity 状态更新
  - **技术方案：** 见 [tech/02_ui_layer.md § 2.3](tech/02_ui_layer.md)
- [ ] **T2.3 自选面板** — 自选行情表（ulist 批量快照）、watchlist.json 本地持久化、涨跌配色、点击选中联动，放入 **stock-app**
  - **产出：** ui/watchlist.rs + watchlist.json | **学习点：** 本地持久化、列表渲染、状态联动
  - **技术方案：** 见 [tech/02_ui_layer.md § 2.4](tech/02_ui_layer.md)
- [ ] **T2.4 K 线图** — gpui-kit CandlestickChart 集成（红涨绿跌 + OHLC tooltip），周期按钮组（1m~W1，无 H4），数据窗口切片自实现缩放/平移，放入 **stock-app**
  - **产出：** ui/chart.rs | **学习点：** CandlestickChart 链式 API、缩放/平移数据窗口模式
  - **技术方案：** 见 [tech/02_ui_layer.md § 2.5](tech/02_ui_layer.md)
- [ ] **T2.5 数据接入与状态管理** — 首次加载（一次性拉取）+ loading/错误态、AppState 状态流转（选中代码/周期/面板数据），放入 **stock-app**
  - **产出：** 完整状态管理 | **学习点：** GUI 状态管理、跨 crate 异步加载
  - **技术方案：** 见 [tech/02_ui_layer.md § 2.6](tech/02_ui_layer.md)

**阶段验收：** 启动显示真实指数卡片/自选/K 线（首次加载为静态数据，轮询在 Phase 3）

---

## Phase 3 — 轮询刷新（~3 天）

> 目标：交易时段感知的轮询服务，UI 实时刷新（完整 MVP）
> 补强：tokio interval、cx.spawn/cx.notify、退避重试

- [ ] **T3.1 交易时段与轮询服务** — TradingPhase 纯函数（09:30/11:30/13:00/15:00 边界）、tokio interval 后台任务（盘中 5s/非交易时段 60s 降频），放入 **stock-data**
  - **产出：** trading_hours.rs + polling.rs | **学习点：** tokio interval、状态机
  - **技术方案：** 见 [tech/01_data_layer.md § 1.6](tech/01_data_layer.md)
- [ ] **T3.2 UI 实时集成** — cx.spawn 消费轮询结果 + cx.notify 刷新、指数/自选/K 线联动刷新、状态栏（轮询状态/最近刷新时间/交易时段），放入 **stock-app**
  - **产出：** 轮询集成 + 状态栏 | **学习点：** gpui cx.spawn、后台任务与 UI 解耦
  - **技术方案：** 见 [tech/02_ui_layer.md § 2.7](tech/02_ui_layer.md)
- [ ] **T3.3 退避与容错** — 网络失败指数退避（1s→2s→4s→30s 上限）、错误态 UI（重试按钮）、限速冲突排队
  - **产出：** 稳定轮询 | **学习点：** 错误恢复、重试模式
  - **技术方案：** 见 [tech/02_ui_layer.md § 2.7](tech/02_ui_layer.md)

**阶段验收：** 盘中启动应用，指数/自选 5s 级刷新，收盘后自动降频，断网恢复自动续传

---

## Phase 4 — 扩展页面（可选，~10 天）

> 目标：对标 stock-dashboard 的进阶页面（按需挑选，非必做）
> 补强：gpui 自绘、指标库迁移、批量计算流水线

- [ ] **T4.1 分时图** — trends2 当日分时 + 均价线（LineChart）+ 昨收基准
- [ ] **T4.2 指标库迁移** — 从 quant 拷贝 SMA/EMA/RSI/MACD/布林带（quant-data/src/indicators），叠加到 K 线图
- [ ] **T4.3 热力图** — squarify treemap 布局纯函数 + gpui 自绘矩形网格，按涨跌幅/换手率着色
- [ ] **T4.4 榜单** — clist 按 fid 排序翻页（涨幅/跌幅/成交额 Tab 切换）
- [ ] **T4.5 搜索与选股扫描** — suggest 搜索加入自选；全市场快照 + 指标信号过滤

**阶段验收：** 热力图渲染全市场涨跌分布，点击格子进入个股详情

---

## 进度追踪表

| 任务 | 状态 | 学习点 | 备注 |
|------|------|--------|------|
| T1.1 | ⬜ | cargo workspace、多 crate 依赖 | workspace 骨架 |
| T1.2 | ⬜ | thiserror、自定义 Deserialize | 模型 + 容错反序列化（stock-data） |
| T1.3 | ⬜ | reqwest 默认头、令牌桶限速 | 东财客户端骨架（stock-data） |
| T1.4 | ⬜ | 分页拼装、CSV 解析、单位契约 | 解析与归一（stock-data） |
| T1.5 | ⬜ | TTL 缓存、fixture 测试 | 缓存 + 测试（stock-data） |
| T2.1 | ⬜ | gpui 启动、主题设计 | 应用骨架 + 语义色（stock-app） |
| T2.2 | ⬜ | 卡片布局、Entity 更新 | 总览面板（stock-app） |
| T2.3 | ⬜ | 本地持久化、列表渲染 | 自选面板（stock-app） |
| T2.4 | ⬜ | CandlestickChart、数据窗口切片 | K 线图（stock-app） |
| T2.5 | ⬜ | GUI 状态管理 | 数据接入 + 状态流转（stock-app） |
| T3.1 | ⬜ | tokio interval、状态机 | 交易时段 + 轮询（stock-data） |
| T3.2 | ⬜ | cx.spawn、cx.notify | UI 实时集成（stock-app） |
| T3.3 | ⬜ | 指数退避、错误恢复 | 退避与容错 |
| T4.1 | ⬜ | LineChart、基准线 | 分时图（可选） |
| T4.2 | ⬜ | 指标迁移、叠加渲染 | 指标库（可选） |
| T4.3 | ⬜ | gpui 自绘、treemap | 热力图（可选） |
| T4.4 | ⬜ | 表格、Tab 切换 | 榜单（可选） |
| T4.5 | ⬜ | suggest 接口、批量指标 | 搜索与扫描（可选） |

**状态说明：** ⬜ 待开始 | 🔵 进行中 | ✅ 已完成 | ⏸ 暂停 | ❌ 已取消
