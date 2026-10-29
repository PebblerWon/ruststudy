# Phase 2-3：界面层技术实现

> gpui-kit 版本以 **0.7.3** 为准（2026-10-15 对照 cargo registry 源码核对过；版本升级后需重新核对）。

## 2.1 模块概述

### 目标

gpui 桌面应用：总览（指数卡片 + 市场统计）+ 自选表 + K 线图 + 状态栏，Phase 3 接入轮询刷新。

### 覆盖的 Rust 知识点

| 知识点 | 实践场景 |
|--------|---------|
| **gpui** | Entity 状态、Render trait、cx.spawn / cx.notify |
| **gpui-kit** | Card/Button/Input 等组件、CandlestickChart、DockArea |
| **状态管理** | AppState 集中管理 + 面板订阅 |
| **异步集成** | 轮询任务与 UI 线程交互 |

## 2.1.1 gpui-kit 0.7.3 API 要点（已核对，写代码前必读）

1. **三层 import 模型**：
   - `use gpui_kit::*;` 只重导出 GPUI 核心 API
   - `v_flex()/h_flex()/StyledExt` 在 `gpui_kit::base::*`
   - 组件库（ActiveTheme/Button/Input/Select/chart/dock/plot）在 `gpui_kit::component::*`
   - `.when()` 需要 `gpui_kit::prelude::FluentBuilder`
2. **CandlestickChart 无内置缩放/平移**：只有 hover tooltip、网格、坐标轴、入场动画；缩放/平移需按"数据窗口切片"模式自实现（§2.5）。
3. `SelectEvent` 只有 `Confirm(Option<Value>)` 变体，没有 `Change`。
4. `InputState::new(window, cx)` 为 builder 风格（`.placeholder()`）；InputEvent 有 Change/PressEnter/Focus/Blur。
5. 项目内 quant 仓库有 vendored `gpui-component`，stock-dashboard 直接用 crates.io 的 gpui-kit 0.7.3 即可（registry 源码可查证 API）。

---

## 2.2 应用骨架与主题（T2.1）

### 启动三段式（与 quant 相同）

```rust
fn main() {
    Application::new().run(|cx| {
        cx.bind_keys([...]);
        let bounds = Bounds::centered(None, size(px(1400.), px(900.)), cx);
        cx.open_window(bounds, |window, cx| {
            cx.new(|cx| StockApp::new(window, cx))
        }).unwrap();
        cx.activate(true);
    });
}
```

### AppState（单页布局；若面板多再切 DockArea）

```rust
pub struct AppState {
    indexes: Vec<IndexQuote>,       // 8 大指数
    snapshot: Option<MarketSnapshot>,
    watchlist: Vec<String>,         // 自选代码
    quotes: Vec<Quote>,             // 自选快照
    selected: Option<String>,       // 选中自选项
    interval: Interval,             // K 线周期
    klines: Vec<Kline>,
    phase: TradingPhase,
    last_refresh: Option<DateTime<Local>>,
    load_state: LoadState,          // Loading / Ready / Error(String)
}
```

### 主题语义色（A 股红涨绿跌）

```rust
// ui/theme.rs
pub const UP_COLOR: Rgba = rgb(0xFF4D4F);     // 涨 — 红
pub const DOWN_COLOR: Rgba = rgb(0x52C41A);   // 跌 — 绿
pub const FLAT_COLOR: Rgba = rgb(0x8C8C8C);
pub fn change_color(pct: f64) -> Rgba {
    if pct > 0. { UP_COLOR } else if pct < 0. { DOWN_COLOR } else { FLAT_COLOR }
}
```

> 显式定义语义色而非复用 gpui-kit 默认涨跌色（其默认为绿涨红跌，加密货币惯例）。

---

## 2.3 总览面板（T2.2）

```
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
│ 上证指数    │ │ 深证成指    │ │ 创业板指    │ │ 科创50     │
│ 3643.75    │ │ 11414.52   │ │ ...        │ │            │
│ -0.39% -14.28│ ...        │            │ │            │
│ 成交 5317亿 │ │            │ │            │ │            │
└────────────┘ └────────────┘ └────────────┘ └────────────┘
```

- 指数卡片：`Card`（gpui_kit::component）+ `v_flex`（名称/价格/涨跌幅/成交额），grid 用两行 `h_flex` 平铺
- 涨跌幅文字 `change_color()` 着色
- 市场统计条：三块（涨跌家数 `up/total`、涨跌停、总成交额），大数字 + 语义色
- 数据：首次加载调 `stock_data::index_quotes()` + `market_snapshot()`

---

## 2.4 自选面板（T2.3）

- 表格列：代码 | 名称 | 现价 | 涨跌幅 | 涨跌额 | 换手率 | 成交额
- gpui-kit 0.7.3 无 Table 组件：用 `h_flex` 表头 + 行列表（`candle` 风格自绘行），列宽固定 + 等宽数字字体（对齐）
- 选中行高亮；点击 → `AppState.selected` 更新 → K 线图联动
- 持久化：`watchlist.json`（`dirs` crate 定位到项目配置目录或工作目录），结构 `{"codes": ["600519", "000001", ...]}`；默认样例自选 3-5 只；启动加载、变更即写

---

## 2.5 K 线图（T2.4）

### 基础渲染

```rust
use gpui_kit::component::chart::CandlestickChart;

CandlestickChart::new("kline")
    .bars(window_slice)          // &[ChartBar] — 开高低收量
    .up_color(UP_COLOR)          // 覆盖默认绿涨
    .down_color(DOWN_COLOR)
    .tooltip_visible(true)       // 内置 OHLC tooltip
```

- 周期按钮组：`ButtonGroup` 或若干 `Button`（1m/5m/15m/30m/60m/日/周；**无 H4**），切换触发重拉
- 复权固定前复权（fqt=1）

### 缩放/平移（数据窗口切片模式，组件不内置）

- 全量 `klines: Vec<Kline>` 常驻内存；渲染只取 `[start, end)` 切片
- 滚轮缩放：调整窗口长度（如 50~500 根），锚定鼠标位置；拖拽平移：移动窗口起点
- 交互事件在包裹 `div` 上监听（`on_scroll` / `on_mouse_down` + `on_mouse_move`），先做最小可用（滚轮缩放 + 拖拽平移 + 右侧贴边）
- 最新一根追加时若窗口贴右缘则自动跟随

### 状态流转

- 周期/代码变更 → `LoadState::Loading` → tokio 任务拉取 → 回填 → `cx.notify()`
- 错误 → `Error(msg)` + 重试按钮

---

## 2.6 数据接入与状态管理（T2.5）

- 首次加载顺序：watchlist.json → 指数 + 市场快照 → 自选快照 → 默认选中第一只的 K 线
- 所有网络调用走 `stock_data`（TTL 缓存兜底），UI 不缓存数据
- `LoadState` 每个面板独立（总览/自选/K 线互不阻塞）

---

## 2.7 轮询集成（Phase 3，T3.2/T3.3）

> ⚠️ 本节下方示例仅示意轮询节奏；**直接在 `cx.spawn` 里 `.await` client 方法在 gpui executor 上不可行**（reqwest 依赖 tokio reactor，会 panic）。实际桥接方案（tokio Runtime + block_on / async-channel）见 [03_integration.md §3.3](03_integration.md)，以该文档为准。

```rust
// AppState::new 中启动（或独立 task，cx.background_spawn）
cx.spawn(async move |this, cx| {
    let mut ticker = tokio::time::interval(poll_interval(phase)); // 盘中 5s / 其余 60s
    loop {
        ticker.tick().await;
        match client.quotes(&codes).await {
            Ok(q) => this.update(cx, |s, cx| {
                s.quotes = q; s.last_refresh = Some(Local::now());
                cx.notify();
            }),
            Err(_) => { /* 退避：ticker 周期 ×2，上限 30s；成功恢复 */ }
        }
    }
});
```

要点：

- 轮询任务只做"拉取 → 更新 Entity → notify"，不做 UI 逻辑
- 三类数据（指数/自选/K 线）可合并为一个任务顺序拉（共享限速桶），或各开一个任务——**建议合并**，天然规避限速冲突
- 退避实现：失败时 `tokio::time::sleep(backoff)`，不用改 interval
- 状态栏显示：`TradingPhase` 图标文案 / 最近刷新时间 / 轮询状态（正常/退避中/停牌时段）

---

## 2.8 手动验收清单

- [ ] 启动 < 3s 显示指数卡片与自选（盘中）
- [ ] 涨跌颜色正确（对照东财网页端）
- [ ] 停牌股显示"停牌"而非 0.00
- [ ] K 线周期切换 < 1s（缓存命中时瞬时）
- [ ] 盘中自选 5s 级跳动；收盘后状态栏显示"已收盘/休市"
- [ ] 断网 30s 不崩溃，恢复后自动续刷
