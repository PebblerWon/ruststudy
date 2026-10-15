# Phase 3：界面层技术实现

## 3.1 模块概述

### 本阶段目标

使用 gpui + gpui-kit（Longbridge 出品，crates.io 正式版）搭建 RustQuant 主界面仪表盘，实现：
- 左侧品种列表侧边栏（WatchlistPanel）、中央 K 线图区域（CandlestickChart 原生组件）
- 右侧指标面板（IndicatorPanel）、顶部工具栏 + 底部状态栏

### 覆盖的 Rust 知识点

| 知识点 | 在 UI 层的应用场景 |
|--------|-------------------|
| gpui Entity/Context 模型 | `cx.new` 创建视图返回 `Entity<T>`，状态变更后 `cx.notify()` 驱动按需重渲染 |
| Render trait | `render()` 每次调用构建元素树（保留模式 UI） |
| Taffy Flexbox 布局 | Tailwind 风格链式样式、`v_flex()` / `h_flex()` 辅助函数 |
| gpui-kit 组件体系 | CandlestickChart、DataTable、Dock、Input、Select、Button、Badge、Theme |
| tokio 桥接基础 | `cx.spawn` 前台任务消费 mpsc、WeakEntity 跨 runtime 更新 UI（Phase 4 深入） |

### 与 Phase 1/2 的衔接

```
Phase 1 (data)          Phase 2 (indicators)       Phase 3 (ui)
┌──────────────┐        ┌──────────────┐          ┌──────────────┐
│  DataFetcher │        │  SMA/EMA/RSI │          │  gpui 界面    │
│  KlineStore  │───────▶│  MACD/BB     │─────────▶│  K 线图渲染   │
│  Kline/Ticker│        │  Indicator   │          │  指标叠加     │
└──────────────┘        └──────────────┘          └──────────────┘
```

- **数据层**提供 `Kline`、`Ticker` → UI 层消费渲染；**指标层**提供 `IndicatorValues` → 叠加曲线
- UI 层不直接调用 REST API，通过 `KlineStore` 获取已缓存数据

---

## 3.2 应用入口与 gpui 配置

### gpui-kit 三层 import 模型（最容易踩的坑）

`use gpui_kit::*;` 只重导出 **GPUI 核心 API**（div、Window、Context、Entity、Task、Hsla...）。
组件库与布局辅助在子模块中，需按路径显式导入：

| 要用的东西 | 正确导入路径 |
|-----------|-------------|
| GPUI 核心（div / Window / Context / Entity / Render / Hsla...） | `use gpui_kit::*;` |
| `v_flex()` / `h_flex()` / `StyledExt` | `use gpui_kit::base::{h_flex, v_flex, StyledExt};` |
| `ActiveTheme`（`cx.theme()`） | `use gpui_kit::component::ActiveTheme;` |
| Button / ButtonGroup | `use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants};` |
| Input / InputState / InputEvent | `use gpui_kit::component::input::{Input, InputState, InputEvent};` |
| Select / SelectState | `use gpui_kit::component::select::{Select, SelectState};` |
| 图表（CandlestickChart / LineChart / BarChart...） | `use gpui_kit::component::chart::*;` |
| Dock / DockArea / Panel / DockLayout / PanelEvent | `use gpui_kit::component::dock::*;` |
| Badge / Plot / IntoPlot 等 | `use gpui_kit::component::badge::*;` / `use gpui_kit::component::plot::*;` |
| `.when(cond, ...)` 链式条件 | `use gpui_kit::prelude::FluentBuilder;`（prelude 中） |

### quant-app/Cargo.toml 依赖配置

```toml
# quant-app/Cargo.toml
[dependencies]
quant-data = { path = "../quant-data" }
# GUI：gpui-kit 重导出全部 GPUI 核心 API（use gpui_kit::*;）；
# 组件库走 gpui_kit::component::*，布局辅助走 gpui_kit::base::*（见 §3.2 import 表）
# 严禁直接添加 gpui 依赖——与 gpui-kit 是两条不兼容的发布线
gpui-kit = "0.7.3"
# 继承 workspace 依赖
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

同步清理：根 `Cargo.toml` 的 `[workspace.dependencies]` 删除 `eframe`、`egui_plot`；gpui-kit 只在 quant-app 声明。

### workspace profile 性能覆盖

```toml
# 根 Cargo.toml（workspace）：debug 下依赖默认 O0，gpui 渲染/字体排版会明显卡顿，
# 为 gpui 栈与字体排版依赖启用 O3（自身代码仍为 O0，保留快速编译与断点调试）
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

> 不想列清单的简化替代：`[profile.dev.package."*"]` + `opt-level = 1`（所有第三方依赖基础优化），
> 效果略差（自身依赖无法到 O3）但免维护；二选一即可。

### 构建要求

MSVC toolchain（DirectX/DirectWrite 后端需 MSVC 链接器）、CMake 在 PATH（部分原生依赖构建需要）、Rust 1.92+（gpui-kit 0.7.3 最低工具链版本）。

### main.rs 启动三段式

```rust
// src/main.rs
use gpui_kit::*;
use gpui_kit::assets::Assets;

fn main() {
    // gpui 启动三段式：application → init → open_window
    gpui_kit::application()
        .with_assets(Assets)              // 内置图标/资源加载
        .run(|cx| {
            gpui_kit::init(cx);           // ① 初始化（必须在开窗前调用）
            gpui_kit::open_window(        // ② 开窗并挂载根视图
                WindowOptions::default(), //    标题/尺寸等配置见官方文档
                cx,
                |_, cx| cx.new(|_| crate::ui::app::QuantApp::new()),
            ).expect("Failed to open window");
        });
}
```

注意：`main` 不是 async——GPUI 有自己的前台执行器，tokio runtime 的启动与桥接见 tech/04 §4.6/§4.7。

### QuantApp 根视图实现 Render trait

```rust
// src/ui/app.rs
use gpui_kit::base::{StyledExt, h_flex, v_flex}; // StyledExt 提供 border_b_1 等扩展样式
use gpui_kit::component::ActiveTheme;
use gpui_kit::*;
use quant_data::common::models::{Kline, Ticker, Interval};

pub struct QuantApp {
    pub state: Entity<AppState>,               // 集中状态（§3.8）
    pub watchlist: Entity<WatchlistPanel>,     // 面板 Entity：render 时插入 Dock 工作区
    pub chart: Entity<ChartPanel>,
    pub indicator_panel: Entity<IndicatorPanel>,
}

impl Render for QuantApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 每次调用重建元素树；只有被 notify 的视图才会真正重绘
        v_flex().size_full()
            .child(self.render_toolbar(window, cx))     // 顶部工具栏（§3.7）
            .child(self.render_dock(window, cx))        // Dock 多面板工作区（§3.3）
            .child(self.render_status_bar(window, cx))  // 底部状态栏（§3.7）
    }
    // render_toolbar / render_dock / render_status_bar 为私有辅助方法（骨架从略）
}
```

### 设计要点

- `init(cx)` 必须在 `open_window` 之前调用（注册组件全局状态/主题）
- `cx.new(|_| QuantApp::new())` 返回 `Entity<QuantApp>`，生命周期由 GPUI 接管；中文字体无需配置（§3.9）

---

## 3.3 主界面布局

### gpui 布局系统与 Dock 工作区

- 布局基于 **Taffy Flexbox**，样式为 **Tailwind 风格链式** API：`div().flex().flex_col().size_full().gap_2()`；gpui-kit 提供 `v_flex()`（纵向）/ `h_flex()`（横向）辅助
- 主工作区使用 gpui-kit **Dock** 组件（可停靠、可拖拽、布局可 serde 持久化）：

```rust
// src/ui/app.rs（render_dock 部分，骨架）
// 面板协议：实现 gpui-kit 的 Panel trait（use gpui_kit::component::dock::*）。
// Panel 继承 gpui-base 的 BasePanel，后者要求 EventEmitter<PanelEvent> + Render + Focusable——
// 即一个面板类型需要：impl Panel + impl EventEmitter<PanelEvent> + impl Render + impl Focusable
// 组装：DockSkin::dock_area(id, version, window, cx) 返回 (Entity<DockArea>, Rc<DockSkin>)，
// DockArea::set_center(DockLayout::h_split()... ) 组合 h_split()（水平）/ v_split()（垂直）/
// tabs()（标签页）构建三区布局；
// 布局树可 serde 持久化到本地，下次启动还原（组合 API 见官方文档 Dock 页）
```

### 布局示意

```
┌──────────────────────────────────────────────────────────┐
│                Toolbar (工具栏)                            │
│  [交易对选择 ▼]  [1m] [5m] [15m] [1h] [4h] [1d]          │
├──────────┬─────────────────────────────┬─────────────────┤
│ Watchlist│      ChartArea (K 线图)      │  IndicatorPanel │
│ (品种列表)│   CandlestickChart          │  (指标面板)      │
│ BTCUSDT  │   gpui-kit 原生组件          │  SMA/EMA/RSI    │
│ ETHUSDT  │   内置 OHLC tooltip/网格     │  MACD/BB        │
├──────────┴─────────────────────────────┴─────────────────┤
│        StatusBar：● 已连接 | BTCUSDT 65,432.10 | 更新      │
└──────────────────────────────────────────────────────────┘
```

### 设计要点

- **保留模式 UI**：Entity 树持久存在，`render()` 按需重执行（对比 egui 每帧全量重绘，§3.9）
- Dock 支持面板拖拽重排与布局持久化；根容器 `v_flex().size_full()` 铺满窗口

---

## 3.4 品种列表侧边栏

### WatchlistPanel：v_flex 列表 + Input 搜索

```rust
// src/ui/watchlist.rs
use gpui_kit::prelude::FluentBuilder;                  // .when(...)
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme;                  // cx.theme()
use gpui_kit::component::input::{InputState, InputEvent};

pub struct WatchlistPanel {
    pub filter_input: Entity<InputState>,  // 搜索框（gpui-kit Input）
    pub filter: String,                    // 过滤关键词（InputEvent::Change 更新）
    pub items: Vec<WatchlistItem>,         // 品种数据（symbol / price / change_24h）
}

impl WatchlistPanel {
    /// 品种行渲染：v_flex 列表（品种少时比 DataTable 更轻量）
    fn render_items(&mut self, state: &Entity<AppState>, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = state.read(cx).selected_symbol.clone();
        v_flex().flex_1().overflow_y_scroll().children(self.filtered_items().map(|item| {
            // 涨跌幅颜色：涨绿跌红（加密货币惯例）
            let color = if item.change_24h >= 0.0 { cx.theme().success } else { cx.theme().danger };
            h_flex().px_2().py_1().gap_2()
                .when(selected == item.symbol, |el| el.bg(cx.theme().primary.opacity(0.15))) // 选中高亮
                .child(item.symbol.clone())
                .child(format!("{:.2}", item.price))
                .child(div().text_color(color).child(format!("{:+.2}%", item.change_24h)))
                .on_click(cx.listener(move |this, _ev, window, cx| this.select_symbol(&item.symbol, window, cx)))
        }))
    }

    /// 选中交易对：state.update 写入 AppState + cx.emit 广播 + cx.notify（§3.8）
    fn select_symbol(&mut self, symbol: &str, _w: &mut Window, cx: &mut Context<Self>) { /* ... */ }
    /// 大小写不敏感过滤
    fn filtered_items(&self) -> impl Iterator<Item = &WatchlistItem> { /* ... */ }
}
```

### 搜索过滤（Input 事件订阅）

```rust
// WatchlistPanel::new 中创建 Input 并订阅变更事件
let filter_input = cx.new(|cx| InputState::new(window, cx).placeholder("搜索交易对..."));
cx.subscribe_in(&filter_input, window, |this, input, event, _window, cx| {
    match event {
        InputEvent::Change => {
            this.filter = input.read(cx).value().to_string();  // 读取当前文本
            cx.notify();                                       // 重渲染列表
        }
        _ => {}
    }
}).detach();
```

### 设计要点

- 品种行 `h_flex()` 横排：名称 / 价格 / 涨跌幅，配色取 `cx.theme().success / .danger`
- 选中状态写入共享 `Entity<AppState>`，图表面板经订阅事件刷新；品种量大时换 DataTable（见 tech/06 §6.6）

---

## 3.5 K 线图渲染（核心）

### gpui-kit CandlestickChart：原生 K 线组件

直接使用 gpui-kit **CandlestickChart** 原生组件。**内置**：hover 自动 OHLC tooltip、网格、
坐标轴标签、入场动画；**不内置缩放/平移**——0.7.3 的 Chart/Plot 层只做 hover 交互，
K 线缩放/平移需自行实现（方案见下）。

```rust
// src/ui/chart.rs
use gpui_kit::*;
use quant_data::common::models::Kline;

/// CandlestickChart 需要的数据点结构
#[derive(Clone)]
pub struct CandleData {
    pub time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// Kline → CandleData 转换（open_time 格式化为坐标轴标签）
pub fn to_candles(klines: &[Kline]) -> Vec<CandleData> {
    klines.iter().map(|k| CandleData {
        time: chrono::DateTime::from_timestamp_millis(k.open_time)
            .map(|dt| dt.format("%m-%d %H:%M").to_string()).unwrap_or_default(),
        open: k.open, high: k.high, low: k.low, close: k.close,
    }).collect()
}
```

### ChartPanel 渲染

```rust
// src/ui/chart.rs
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::base::div;
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::chart::CandlestickChart;

impl Render for ChartPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let candles = to_candles(&self.klines);
        if candles.is_empty() {
            // 早返回与正常返回是两个不同类型，必须 into_any_element() 抹平
            return div().size_full().flex().items_center().justify_center()
                .child("暂无 K 线数据").into_any_element();
        }
        // 访问器闭包描述"怎么读数据"；tooltip/网格/坐标轴组件内置
        CandlestickChart::new(candles)
            .x(|d| d.time.clone())
            .open(|d| d.open).high(|d| d.high).low(|d| d.low).close(|d| d.close)
            .body_width_ratio(0.4)          // 蜡烛实体宽度占格子比例（默认 0.8）
            .bullish(cx.theme().success)    // 阳线：绿涨（加密货币惯例）
            .bearish(cx.theme().danger)     // 阴线：红跌
            .into_any_element()
        // 鼠标悬停自动渲染 OHLC tooltip，无需任何绘制代码
    }
}
```

### 缩放/平移：自行实现（组件不提供）

组件无内置 zoom/pan，Phase 3 可先交付静态渲染，缩放/平移作为增强按"数据窗口"模式实现：

1. ChartPanel 增加视口状态字段：`visible_start: usize, visible_len: usize`
2. `on_scroll_wheel`（`cx.listener`）按滚轮调整 `visible_len`（缩放）/ 配合按键或拖拽平移 `visible_start`
3. `render` 时只把 `&klines[visible_start..visible_start+visible_len]` 传给 CandlestickChart——
   组件按传入数据自适应坐标轴，天然完成"视图变换"
4. 状态变更后记得 `cx.notify()`

### 指标曲线叠加

1. **Plot 层自定义**（参考线/自定义元素）：`#[derive(IntoPlot)]` 实现 `Plot` trait，用 `ScaleLinear`（值域→像素域线性映射）把指标值画进图表坐标系
2. **LineChart 组件**：MA 类单线指标用同系列 `LineChart`，色板取 `cx.theme().chart_1..5`

```rust
// 方式 1 骨架
#[derive(IntoPlot)]
struct IndicatorOverlay {
    values: Vec<f64>,   // 指标值序列（与 K 线索引对齐）
    color: Hsla,        // 曲线颜色（cx.theme().chart_1..5）
}
impl Plot for IndicatorOverlay {
    // ScaleLinear 将指标值域映射到图表像素域，逐点连线（详见官方文档 Plot 页）
}
```

### 与 egui 手绘方案的对比

| 能力 | egui Shape API 手绘 | gpui-kit CandlestickChart |
|------|--------------------|--------------------------|
| OHLC tooltip / 网格 / 坐标轴 | 自行实现 | 内置 |
| 缩放 / 平移 | 手写仿射映射 + `handle_zoom` / `handle_pan` | 组件不提供，按"数据窗口"模式自实现（见上） |
| K 线部分代码量 | ~300 行 | ~30 行（不含自实现缩放/平移） |

### 设计要点

- 访问器闭包把数据结构与组件解耦；涨跌配色遵循加密货币惯例（bullish 绿 / bearish 红）
- `#[derive(IntoPlot)]` 会为类型生成 `IntoElement`（包一层 `PlotElement`），自定义 Plot 可直接 `.child()` 进元素树
- 同系列 `LineChart` / `BarChart` / `AreaChart` 供指标面板复用

---

## 3.6 指标面板

```rust
// src/ui/indicator_panel.rs
#[derive(Clone, Copy, PartialEq)]
pub enum ActiveIndicator { Sma, Ema, Rsi, Macd, Bollinger }

pub struct IndicatorPanel {
    pub active: ActiveIndicator,
    // 指标参数（Input 输入，变更时校验后重算）
    pub sma_period: usize,  pub ema_period: usize,  pub rsi_period: usize,
    pub macd_fast: usize,   pub macd_slow: usize,   pub macd_signal: usize,
    pub bb_period: usize,   pub bb_std_dev: f64,
}
```

曲线绘制（LineChart + chart 色板）：

| 指标 | 曲线 | 色板 |
|------|------|------|
| SMA / EMA | 单线 | `chart_1` / `chart_2` |
| RSI | 单线（70/30 参考线经 Plot 层叠加） | `chart_3` |
| MACD | DIF/DEA 双线 + 柱（柱用 BarChart） | `chart_4` / `chart_5` |
| BB | 上/中/下三线 | `chart_1..3` |

```rust
// 指标切换：Select 组件（或 Button 组），选中后 cx.notify() 重绘曲线区
// 参数调整：Input + 数值校验（validate），订阅模式同 §3.4 搜索框：
//   InputEvent::Change → 解析数值（非法标红不生效）→ 写入参数
//   → state.update 触发 recompute_indicators() → cx.notify()
```

### 设计要点

- 指标计算仍调用指标层（Phase 2）的 `Indicator` 实现，UI 面板只负责"参数 → 曲线"映射
- 参数变更统一走 `AppState.recompute_indicators()`，避免面板各自持有计算副本

---

## 3.7 工具栏与状态栏

```rust
// 工具栏：交易对选择 + 周期切换（h_flex 单行布局）
fn render_toolbar(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state = self.state.read(cx);
    h_flex().px_3().py_2().gap_3().items_center()
        .border_b_1().border_color(cx.theme().border)
        // 交易对选择：Select/Combobox 组件，on_change 回调（cx.listener）写入 AppState 并广播事件
        .child(/* Select：选项 = state.available_symbols，当前 = selected_symbol */)
        // K 线周期切换：Button 组，选中项高亮 primary 色
        .children([Interval::M1, Interval::M5, Interval::M15,
                   Interval::H1, Interval::H4, Interval::D1].map(|iv| {
            // Button::new(iv.as_str()).label(iv.as_str())
            //     .when(state.current_interval == iv, 高亮样式)
            //     .on_click(cx.listener(...)) → 切换周期 + 触发数据加载
        }))
}

// 状态栏：连接状态（Badge）+ 最后更新时间 + 数据条数（h_flex 单行布局）
fn render_status_bar(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state = self.state.read(cx);
    let last_update = state.last_update_time
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|dt| dt.format("%H:%M:%S").to_string()).unwrap_or_default();
    h_flex().px_3().py_1().gap_4().items_center()
        .border_t_1().border_color(cx.theme().border)
        .child(/* Badge：绿=已连接 / 红=未连接，色取 success/danger */)
        .child(format!("BTCUSDT: {:.2}", /* latest_ticker 价格 */))
        .child(format!("更新: {}", last_update))
        .child(format!("K 线: {} 条", state.current_klines.len()))
}
```

### 设计要点

- Badge / Select / Button 的具体构造参数见官方文档对应组件页；时间格式化复用 `chrono`
- **组件状态 Entity 绝不能在 `render()` 里创建**：`SelectState` / `InputState` 这类有状态的
  Entity 必须在 `new(window, cx)` 中 `cx.new(...)` 创建并存入字段，`render()` 只引用。
  在 render 里 `cx.new(|cx| SelectState::new(...))` 每次重渲染都会重建 Entity——选中值、
  焦点、下拉展开状态全部丢失，且旧 Entity 泄漏。这是 gpui 从即时模式迁移过来最容易犯的错
- Select 的事件订阅（`SelectEvent::Confirm(Option<Value>)`，确认选择时发出）模式与 §3.4
  搜索框的 `InputEvent::Change` 相同，用 `cx.subscribe_in` + `.detach()`

---

## 3.8 应用状态管理

### AppState 作为 Entity

```rust
// src/ui/app_state.rs
use gpui_kit::*;
use quant_data::common::models::{Kline, Ticker, Interval, IndicatorValues};

/// AppState 变化事件（面板间通信载体）
#[derive(Clone)]
pub enum AppStateChanged { SymbolSelected(SharedString), IntervalChanged(Interval) }

/// 集中管理所有 UI 状态（作为 Entity 被各面板 observe/subscribe）
pub struct AppState {
    // === 交易对与周期 ===
    pub selected_symbol: SharedString,
    pub current_interval: Interval,
    pub available_symbols: Vec<SharedString>,
    // === K 线与指标 ===
    pub current_klines: Vec<Kline>,
    pub indicator_values: Option<IndicatorValues>,
    // === 实时数据（Phase 4 填充）===
    pub latest_ticker: Option<Ticker>,
    pub is_connected: bool,
    pub last_update_time: Option<i64>,
}
impl EventEmitter<AppStateChanged> for AppState {}

impl AppState {
    pub fn new() -> Self { /* 默认 BTCUSDT / M15，品种列表初始硬编码 */ }

    /// 加载当前交易对和周期的 K 线数据（从 KlineStore）
    pub fn load_kline_data(&mut self, cx: &mut Context<Self>) {
        // Phase 3: 同步加载；Phase 4: 实时增量经 mpsc → cx.spawn 消费循环（tech/04 §4.6）
        self.recompute_indicators();
        cx.notify();   // 忘记这行 = UI 不更新
    }
    /// 重新计算指标（调用指标层，逻辑与原方案一致）
    pub fn recompute_indicators(&mut self) { /* ... */ }
}
```

### 面板间通信：observe / subscribe

```rust
// 各面板在 new 时建立与 AppState 的连接（以 ChartPanel 为例）
// 方式 1：observe——AppState 任何 notify 都触发本面板重渲染（粗粒度）
cx.observe(&state, |_this, _state, cx| cx.notify()).detach();

// 方式 2：subscribe_in——只响应关注的事件（细粒度，面板间通信标准模式）
cx.subscribe_in(&state, window, |this, _state, event, _window, cx| {
    match event {
        AppStateChanged::SymbolSelected(symbol) => {
            this.reload_candles(symbol);   // 选中品种变化 → 图表刷新
            cx.notify();
        }
        AppStateChanged::IntervalChanged(_) => { /* 同上 */ }
    }
}).detach();   // Subscription 需 detach，随宿主 Entity 释放自动清理

// 事件发送端（如品种列表点击后）：
state.update(cx, |s, cx| {
    s.selected_symbol = symbol.into();
    s.load_kline_data(cx);
    cx.emit(AppStateChanged::SymbolSelected(symbol.into())); // 广播给订阅者
    cx.notify();                                             // AppState 自身重渲染
});
```

### Entity 树与状态更新流程

```
QuantApp (root) ─ Entity<AppState>（共享状态）＋ Entity<WatchlistPanel / ChartPanel / IndicatorPanel>
                  （面板均 observe / subscribe AppState，互不直接引用）
```

```
用户操作 → cx.listener → state.update → load_kline_data() → KlineStore.get()
    → recompute_indicators() → cx.emit(AppStateChanged) + cx.notify()
    → 订阅面板回调刷新自身 → GPUI 按需重渲染（只重绘受影响视图）
```

### 设计要点

- **状态变更三步**：改字段 → `cx.emit(...)`（可选，通知他人）→ `cx.notify()`（驱动重渲染）
- `available_symbols` 初始硬编码，后续可从 `GET /api/v3/exchangeInfo` 动态获取

---

## 3.9 设计要点

### gpui 心智模型：notify 驱动按需重渲染

| 维度 | egui（即时模式） | gpui（保留模式） |
|------|-----------------|-----------------|
| 渲染时机 | `update()` 每帧调用（30-60 FPS） | `render()` 仅在 Entity 被 notify / 输入事件后执行 |
| UI 树 / 状态 | 每帧重建、widget 状态靠 `Id` 存储 | Entity 树持久存在，状态就是 Entity 字段 |
| 刷新触发 | 改状态后下一帧自动反映 | 必须显式 `cx.notify()`，否则 UI 不更新 |

### cx.notify() 纪律（最常见 bug）

- 忘记调用 = 数据变了但界面不动，**没有任何报错**；建议写在状态更新方法的最后一行形成肌肉记忆

### Task 句柄存字段（防 drop 取消）

- `cx.spawn` 返回的 `Task` 被 drop 时任务即被取消；长期运行的前台任务（如 Phase 4 通道消费循环）
  必须存入字段 `_recv_task: Task<()>`（下划线前缀表达"仅持有防取消"）

### 中文字体：系统字体直用

- Windows 上经 DirectWrite 使用系统 UI 字体，默认 `.SystemUIFont`（中文 Windows 即微软雅黑），
  **无需**加载字体文件——egui 时代的 `setup_fonts` / `include_bytes!` 代码全部删除

### 官方文档

- gpui-kit 文档：<https://gpui-kit.com>（含中文 zh-CN）；Select / Button / Badge / Dock / Plot 的
  构造参数以文档为准，本文档骨架保证核心交互模式（Entity / notify / listener / subscribe）准确
- import 路径速查见 §3.2 "三层 import 模型"表；编译报 "cannot find function `v_flex`" /
  "no method named `theme`" / "no method named `when`" 时，99% 是漏了对应子模块导入
