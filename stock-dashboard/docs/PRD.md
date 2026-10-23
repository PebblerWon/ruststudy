# Stock Dashboard — A 股行情看板（Rust 桌面版）

## 1. 项目概述

面向 Rust 初学者的 A 股行情看板学习项目，构建市场总览 + 自选管理 + K 线图表的桌面应用，产品形态对标 [chengzuopeng/stock-dashboard](https://github.com/chengzuopeng/stock-dashboard)（React 版），技术架构参照本仓库 quant 项目的分层模式。

**学习衔接：** myapp（CLI 基础）→ rustkv（并发/异步/高级 trait）→ quant（GUI/实时/回测）→ Stock Dashboard（GUI 实战：真实数据源 + 看板产品形态）

**核心价值：**
- gpui + gpui-kit 桌面应用实战（从 mock 数据升级到真实行情数据源）
- 异步 Rust 实战（tokio + reqwest 轮询、TTL 缓存、令牌桶限速）
- 数据工程（serde 容错反序列化、单位归一契约、本地缓存）
- 数据可视化（指数卡片、行情列表、K 线图，可选热力图自绘）

**技术栈：** gpui + gpui-kit (GUI) | tokio + reqwest (异步/HTTP 轮询) | serde (序列化) | polars (K 线缓存，可选) | thiserror (错误)

## 2. 目标用户

Rust 初学者（项目所有者），前端背景，已完成 myapp + rustkv，quant 项目进行中。代码由项目所有者本人编写，AI 负责文档维护。

## 3. 数据源

东方财富公开行情接口（非官方，HTTP + JSON）：

| 接口 | 用途 |
|------|------|
| `push2.eastmoney.com/api/qt/clist/get` | 全市场行情列表（总览统计/热力图/榜单） |
| `push2.eastmoney.com/api/qt/ulist.np/get` | 批量快照（指数卡片/自选刷新） |
| `push2his.eastmoney.com/api/qt/stock/kline/get` | 历史 K 线（日线全量，分钟线约 32 个交易日） |
| `push2his.eastmoney.com/api/qt/stock/trends2/get` | 当日分时（含均价线） |

> 所有接口规格均经 2026-10-23 实测验证，字段映射与已知限制详见 [tech/01_data_layer.md](tech/01_data_layer.md)。

## 4. 功能需求

### 4.1 P0 核心功能（Phase 1-3 交付，MVP）

- **F1 市场总览**：8 大指数卡片（上证指数/深证成指/创业板指/科创50/沪深300/上证50/中证500/北证50）+ 全市场涨跌家数统计 + 涨跌停 + 总成交额
  - 验收：指数卡片显示实时价格/涨跌幅/成交额；涨跌统计与东财网页端一致
- **F2 自选行情**：自选股批量快照（代码/名称/现价/涨跌幅/换手率/成交额），自选列表本地 JSON 持久化
  - 验收：自选项点击后 K 线图联动；重启后自选列表保留
- **F3 个股 K 线**：多周期（1m/5m/15m/30m/60m/日/周），前复权，红涨绿跌，OHLC tooltip
  - 验收：OHLC 数据与东财网页端一致；周期切换流畅；有 loading/错误态
- **F4 轮询刷新**：行情/指数 5s、全市场快照 60s、K 线 10min 惰性加载；非交易时段自动降频 60s
  - 验收：盘中数据 5s 级刷新；收盘后自动降频；网络异常退避重试不崩溃

### 4.2 P1 扩展功能（Phase 4 可选交付）

- **F5 个股详情/分时**：分时图（trends2 + 均价线）+ 关键指标（换手/市值/振幅）
- **F6 热力图**：全市场/行业/自选维度 treemap，按涨跌幅或换手率着色（gpui 自绘）
- **F7 榜单**：涨幅榜/跌幅榜/成交额榜（clist 排序翻页）
- **F8 搜索添加自选**：代码/名称搜索（suggest 接口），加入自选
- **F9 选股扫描**：全市场快照 + 指标信号过滤（MACD 金叉、布林突破，指标库从 quant 迁移）

### 4.3 阶段交付物

**Phase 1 — 数据层**

- `stock-data` crate：EastmoneyClient（批量快照/指数/全市场/K 线/分时）+ 单位归一 + TTL 缓存
- secid 市场规则、交易时段判断等纯函数及单测
- CLI 验证：`cargo run -p stock-app -- fetch --code 600519` 打印 K 线表格

**Phase 2 — 界面层**

- `stock-app` crate：gpui 桌面应用，总览面板（指数卡片 + 市场统计）+ 自选面板 + K 线图
- 主题语义色：A 股红涨绿跌

**Phase 3 — 轮询刷新**

- TradingPhase 交易时段 + tokio interval 轮询服务
- UI 实时集成（cx.spawn + cx.notify）+ 状态栏

**Phase 4 — 扩展页面（可选）**

- 分时图 / 热力图 / 榜单 / 搜索 / 选股扫描

## 5. 数据模型

```rust
// 个股实时快照（clist / ulist 归一后；单位契约：市值亿元、成交额万元、成交量手）
Quote { code, name, market, price, change, change_pct, volume, amount, turnover_rate, open, high, low, pre_close, total_mv, circ_mv }

// 指数快照（总览页卡片）
IndexQuote { code, name, price, change, change_pct, amount }

// K 线（与 quant 的 Kline 模型一致，可直接拷贝）
Kline { open_time, open, high, low, close, volume, close_time, quote_volume, trades_count, is_closed }

// 分时点
TimelinePoint { time, price, avg_price, volume, amount }

// 全市场快照统计（总览页）
MarketSnapshot { total, up_count, down_count, flat_count, limit_up, limit_down, total_amount }
```

## 6. 非功能需求

| 类别 | 要求 |
|------|------|
| 性能 | K 线图 1000 根 > 30fps；全市场快照解析（~5400 条）< 200ms |
| 实时性 | 盘中行情轮询 5s；UI 刷新帧率 > 30fps |
| 错误处理 | 返回 Result 不 panic；网络异常退避重试；UI 展示错误态 |
| 礼貌性 | 客户端限速 ≥500ms/请求；携带 UA + Referer；仅学习用途 |
| 跨平台 | Windows / macOS / Linux |

## 7. UI 布局

参照 stock-dashboard 首页（https://chengzuopeng.github.io/stock-dashboard/）：

```
┌──────────────────────────────────────────────────┐
│ 顶部：搜索框（Phase 4）                           │
├──────────────────────────────────────────────────┤
│ [上证指数] [深证成指] [创业板指] [科创50]          │  ← 指数卡片 grid
│ [沪深300] [上证50]  [中证500] [北证50]            │
├──────────────────────────────────────────────────┤
│ [市场涨跌 641/4872] [涨跌停 22/82] [全市场成交额] │  ← market_snapshot
├──────────────────────────────────────────────────┤
│ 自选行情表（现价/涨跌幅/换手率/成交额）            │  ← 5s 轮询
├──────────────────────────────────────────────────┤
│ K 线图（选中自选项，多周期切换）                   │
├──────────────────────────────────────────────────┤
│ 状态栏：轮询状态 | 最近刷新 | 交易时段 | 数据源    │
└──────────────────────────────────────────────────┘
```

**配色决策：** A 股惯例**红涨绿跌**（`#FF4D4F` 涨 / `#52C41A` 跌），主题中作为语义色（`up_color`/`down_color`）定义，与 quant 的 crypto 面板（绿涨红跌）明确区分。

## 8. 约束与假设

- 东财接口非官方：需携带 UA + Referer 请求头（实测缺失返回空），客户端限速，仅学习用途
- 东财无公开 WebSocket，实时性采用轮询；分钟线历史仅最近 ~32 个交易日（日线全量），分钟级数据靠本地累积
- 北交所暂不支持（实测数据异常）；东财无 4h K 线
- 单用户桌面；指标库不自行实现（需要时从 quant 项目拷贝 SMA/EMA 等）
- 自选列表 MVP 不做云同步，本地 JSON 即可
