# Stock Dashboard 架构设计

## 1. 架构概述

### 1.1 设计目标

| 目标 | 说明 |
|------|------|
| **复用 quant 模式** | 分层架构、trait 抽象、mock 可测——沿用 quant 已验证的结构 |
| **真实数据源** | 东财公开接口，从第一天起 CLI 验证真实数据，不留 mock 层 |
| **轮询式实时** | 无 WebSocket，用分级轮询（5s/60s）+ 交易时段感知实现"准实时" |

### 1.2 架构风格

**分层架构 + 轮询驱动**：

- **分层架构**：`data → ui` 单向依赖；`common` 被所有层依赖
- **轮询驱动**：后台 tokio 任务定时拉取 → 经 TTL 缓存去重 → channel/Entity 更新 → UI `cx.notify()` 响应

与 quant 的差异：quant 实时层是 WebSocket 推送（mpsc channel），本项目是轮询拉取（interval task）。对 UI 而言消费模式一致——都是"异步任务更新状态 + notify"，channel 可省略（轮询任务直接拿结果回 `cx.update`），或保留 channel 以便复用 quant 的 UI 集成模式（二选一，Phase 3 时定）。

---

## 2. 系统分层

```
┌─────────────────────────────────────────────────┐
│               界面层 (ui)                        │
│   总览面板 / 自选面板 / K 线图 / 状态栏           │
├─────────────────────────────────────────────────┤
│               数据层 (data)                      │
│   EastmoneyClient / TTL 缓存 / 轮询服务          │
├─────────────────────────────────────────────────┤
│               公共层 (common)                    │
│   模型(Quote/Kline/...) / 错误 / 交易时段        │
└─────────────────────────────────────────────────┘
依赖方向：ui → data → common
```

| 层 | 模块 | 职责 | 关键依赖 |
|----|------|------|---------|
| **公共层** | `common` | 数据模型、错误类型、交易时段判断 | serde, thiserror, chrono |
| **数据层** | `data` | 东财接口调用、解析归一、TTL 缓存、轮询调度 | reqwest, tokio |
| **界面层** | `ui` | 总览/自选/K 线渲染、状态管理 | gpui, gpui-kit |

---

## 3. 目录结构

```
stock-dashboard/
├── Cargo.toml                  # workspace
├── crates/
│   ├── stock-data/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── common/
│   │       │   ├── mod.rs
│   │       │   ├── error.rs        # DataError (thiserror)
│   │       │   ├── models.rs       # Quote/IndexQuote/Kline/TimelinePoint/MarketSnapshot
│   │       │   └── trading_hours.rs # TradingPhase 交易时段纯函数
│   │       └── data/
│   │           ├── mod.rs
│   │           ├── eastmoney.rs        # EastmoneyClient（请求头/限速/secid）
│   │           ├── eastmoney_types.rs  # 东财 raw 响应类型（"-" 容错）
│   │           ├── normalize.rs        # 单位归一（元→万/亿、手→股）
│   │           ├── cache.rs            # 内存 TTL 缓存
│   │           └── polling.rs          # tokio interval 轮询服务
│   └── stock-app/
│       └── src/
│           ├── main.rs
│           └── ui/
│               ├── app.rs          # 主窗口 + AppState
│               ├── theme.rs        # A 股语义色（红涨绿跌）
│               ├── overview.rs     # 总览：指数卡片 + 市场统计
│               ├── watchlist.rs    # 自选表 + watchlist.json
│               ├── chart.rs        # K 线图（CandlestickChart）
│               └── status_bar.rs   # 状态栏
├── docs/
│   ├── PRD.md
│   ├── DEV_PLAN.md
│   ├── ARCHITECTURE.md
│   └── tech/
│       ├── 01_data_layer.md
│       └── 02_ui_layer.md
```

> 注：`crates/` 子目录组织或根目录平铺（与 quant 一致）二选一，T1.1 时定。

---

## 4. 数据流

```
东财接口 (HTTP+JSON)
    │  reqwest（UA+Referer 头、令牌桶限速 ≥500ms）
    ▼
EastmoneyClient ──解析──▶ eastmoney_types (raw)
    │                        │ normalize 归一（单位契约：亿/万/手）
    ▼                        ▼
TTL 缓存（行情5s/K线10min/快照60s）──▶ Quote/IndexQuote/Kline/TimelinePoint
    │
    ▼
轮询服务（tokio interval，盘中5s/收盘60s）
    │
    ▼
stock-app: cx.spawn 消费 → 更新 Entity<AppState> → cx.notify()
    │
    ▼
ui/: overview / watchlist / chart / status_bar 重渲染
```

### 4.1 单位契约

归一在 client 出口一次完成，UI 层不得再换算：**市值 → 亿元、成交额 → 万元、成交量保留手**。raw 层保留东财原始单位（元/手）。

### 4.2 缓存分级

| 数据 | TTL | 消费方 |
|------|-----|--------|
| 个股/自选/指数行情 | 5s | 总览、自选 |
| K 线（历史） | 10min | 图表 |
| 全市场快照 | 60s | 总览统计、（Phase 4）热力图/榜单 |

---

## 5. 关键设计决策

| 决策 | 结论 | 理由 |
|------|------|------|
| 配色 | 红涨绿跌（语义色 up/down） | A 股市场惯例；与 quant 的 crypto 面板相反，theme.rs 显式定义避免混淆 |
| H4 周期 | 不支持 | 东财无 4h K 线；M1~W1 共 7 档 |
| 北交所 | 不支持 | 实测数据异常（占位零值）；secid 层拒绝 8/4/9 开头 |
| 分钟历史 | 本地累积 | 接口仅 ~32 个交易日；日线全量够 MVP；如需长历史分钟线，从 quant 迁移 KlineStore 或接 Binance |
| 指标计算 | 不自行实现 | 需要时从 quant 拷贝 indicators 模块（SMA/EMA/RSI/MACD/布林带已验证） |
| K 线缩放/平移 | 数据窗口切片自实现 | gpui-kit CandlestickChart 0.7.3 无内置缩放/平移（hover tooltip/网格/坐标轴有） |

---

## 6. 测试策略

| 层 | 方式 |
|----|------|
| 纯函数 | secid 规则、normalize 单位换算、TradingPhase 边界、CSV 解析（含前复权负价）——普通单测 |
| HTTP 解析 | wiremock/fixture（`tests/fixtures/eastmoney/*.json`，取自 tech/01 实测样例） |
| 真实接口 | `#[ignore]` smoke test（手动 `cargo test -- --ignored`） |
| UI | 手动验收（DEV_PLAN 各阶段验收标准） |
