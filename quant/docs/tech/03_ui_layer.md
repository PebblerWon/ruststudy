# Phase 3：界面层技术实现

## 3.1 模块概述

### 本阶段目标

使用 egui + eframe 搭建 RustQuant 主界面仪表盘，实现：
- 左侧品种列表侧边栏（WatchlistPanel）
- 中央 K 线图区域（CandlestickChart）
- 底部指标面板（IndicatorPanel）
- 顶部工具栏 + 底部状态栏

### 覆盖的 Rust 知识点

| 知识点 | 在 UI 层的应用场景 |
|--------|-------------------|
| egui 即时模式 GUI | 每帧重绘心智模型、`ui.horizontal()`/`ui.vertical()` 布局 |
| eframe::App trait | 应用生命周期管理、`update()` 回调 |
| 状态管理 | `AppState` 集中管理 UI 状态、`Arc<Mutex<>>` 跨线程共享 |
| 自定义绘制 | egui `Shape` API（`RectShape`、`LineShape`）绘制 K 线 |
| 坐标变换 | 价格→像素、时间→像素的仿射映射 |

### 与 Phase 1/2 的衔接

```
Phase 1 (data)          Phase 2 (indicators)       Phase 3 (ui)
┌──────────────┐        ┌──────────────┐          ┌──────────────┐
│  DataFetcher │        │  SMA/EMA/RSI │          │  egui 界面    │
│  KlineStore  │───────▶│  MACD/BB     │─────────▶│  K 线图渲染   │
│  Kline/Ticker│        │  Indicator   │          │  指标叠加     │
└──────────────┘        └──────────────┘          └──────────────┘
```

- **数据层**提供 `Kline`、`Ticker` 数据 → UI 层消费并渲染
- **指标层**提供 `IndicatorValues` → UI 层叠加曲线
- UI 层不直接调用 REST API，通过 `KlineStore` 获取已缓存数据

---

## 3.2 应用入口与 eframe 配置

### main.rs 启动配置

```rust
// src/main.rs
use eframe::NativeOptions;
use rustquant::ui::QuantApp;

#[tokio::main]
async fn main() -> eframe::Result {
    // 初始化 tracing 日志
    tracing_subscriber::fmt()
        .with_env_filter("rustquant=info")
        .init();

    // eframe 窗口配置
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RustQuant — 桌面量化交易系统")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    // 启动 egui 应用
    eframe::run_native(
        "RustQuant",
        options,
        Box::new(|cc| {
            // 加载自定义中文字体
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(QuantApp::new(cc)))
        }),
    )
}

/// 加载中文字体（egui 默认字体不含中文）
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "my_chinese_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/SourceHanSansSC-Regular.otf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_chinese_font".to_owned());
}
```

### 设计要点

- `#[tokio::main]` 启动异步运行时，eframe 内部会复用此 runtime
- 中文字体必须手动加载，否则中文显示为方块
- `ViewportBuilder` 配置窗口标题、默认尺寸、最小尺寸
- `Box::new(|cc| ...)` 闭包在应用启动时执行一次，用于初始化 `QuantApp`

---

## 3.3 主界面布局

### Panel 系统划分

```
┌──────────────────────────────────────────────────────────┐
│                    TopPanel (工具栏)                       │
│  [交易对选择 ▼]  [1m] [5m] [15m] [1h] [4h] [1d]          │
├──────────┬───────────────────────────────────────────────┤
│          │                                               │
│ SidePanel│           CentralPanel                        │
│ (品种列表)│          (K 线图区域)                          │
│          │                                               │
│ BTCUSDT  │    ┌─────────────────────────────────┐        │
│ ETHUSDT  │    │     CandlestickChart            │        │
│ BNBUSDT  │    │     (自定义 Shape 绘制)           │        │
│ SOLUSDT  │    │                                 │        │
│ ...      │    └─────────────────────────────────┘        │
│          │                                               │
│          ├───────────────────────────────────────────────┤
│          │           BottomPanel (指标面板)               │
│          │    [SMA] [EMA] [RSI] [MACD] [BB]              │
│          │    ┌─────────────────────────────────┐        │
│          │    │     IndicatorPanel              │        │
│          │    └─────────────────────────────────┘        │
├──────────┴───────────────────────────────────────────────┤
│                    StatusBar (状态栏)                      │
│  ● 已连接  |  BTCUSDT: 65,432.10  |  更新: 12:34:56     │
└──────────────────────────────────────────────────────────┘
```

### 布局代码骨架

```rust
// src/ui/app.rs
use eframe::egui;

pub struct QuantApp {
    state: AppState,
    // 子面板
    watchlist: WatchlistPanel,
    chart: CandlestickChart,
    indicator_panel: IndicatorPanel,
}

impl eframe::App for QuantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 顶部工具栏
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.toolbar_ui(ui);
        });

        // 底部状态栏
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.status_bar_ui(ui);
        });

        // 底部指标面板
        egui::TopBottomPanel::bottom("indicator_panel")
            .default_height(200.0)
            .show(ctx, |ui| {
                self.indicator_panel.ui(ui, &mut self.state);
            });

        // 左侧品种列表
        egui::SidePanel::left("watchlist")
            .default_width(200.0)
            .show(ctx, |ui| {
                self.watchlist.ui(ui, &mut self.state);
            });

        // 中央 K 线图
        egui::CentralPanel::default().show(ctx, |ui| {
            self.chart.ui(ui, &mut self.state);
        });
    }
}
```

### 设计要点

- Panel 添加顺序影响布局：先添加 `TopBottomPanel`，再添加 `SidePanel`，最后 `CentralPanel` 填充剩余空间
- `default_height` / `default_width` 设置面板默认尺寸，用户可拖拽调整
- 即时模式：`update()` 每帧调用，所有 UI 状态在调用间保持

---

## 3.4 品种列表侧边栏

### WatchlistPanel 结构体

```rust
// src/ui/control_panel.rs
use egui::{Ui, Color32, RichText};

pub struct WatchlistPanel {
    /// 搜索过滤关键词
    filter: String,
    /// 品种列表数据（从 data 层获取）
    symbols: Vec<WatchlistItem>,
}

pub struct WatchlistItem {
    pub symbol: String,
    pub price: f64,
    pub change_24h: f64,  // 24h 涨跌幅百分比
}

impl WatchlistPanel {
    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        // 搜索框
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.filter);
        });
        ui.separator();

        // 表头
        ui.horizontal(|ui| {
            ui.strong("交易对");
            ui.add_space(ui.available_width() - 120.0);
            ui.strong("价格");
            ui.strong("24h%");
        });
        ui.separator();

        // 品种列表（可滚动区域）
        egui::ScrollArea::vertical().show(ui, |ui| {
            for item in self.filtered_items() {
                let is_selected = state.selected_symbol == item.symbol;
                let bg_color = if is_selected {
                    ui.visuals().selection.bg_fill
                } else {
                    Color32::TRANSPARENT
                };

                ui.horizontal(|ui| {
                    // 点击选中交易对
                    let response = ui
                        .scope(|ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&item.symbol).strong());
                                ui.add_space(ui.available_width() - 120.0);
                                ui.label(format!("{:.2}", item.price));
                                // 涨跌幅颜色：涨绿跌红
                                let change_color = if item.change_24h >= 0.0 {
                                    Color32::from_rgb(0, 200, 83)
                                } else {
                                    Color32::from_rgb(234, 57, 67)
                                };
                                ui.label(
                                    RichText::new(format!("{:+.2}%", item.change_24h))
                                        .color(change_color),
                                );
                            })
                        })
                        .response;

                    if response.clicked() {
                        state.selected_symbol = item.symbol.clone();
                        state.load_kline_data();  // 触发数据加载
                    }
                });
            }
        });
    }

    /// 根据搜索过滤返回品种列表
    fn filtered_items(&self) -> Vec<&WatchlistItem> {
        if self.filter.is_empty() {
            self.symbols.iter().collect()
        } else {
            self.symbols
                .iter()
                .filter(|item| item.symbol.to_lowercase().contains(&self.filter.to_lowercase()))
                .collect()
        }
    }
}
```

### 设计要点

- `ScrollArea::vertical()` 使品种列表可滚动，避免品种过多时溢出
- 搜索过滤使用大小写不敏感匹配
- 选中状态通过 `AppState.selected_symbol` 管理，点击后触发 K 线数据加载
- 涨跌幅颜色：涨绿 (`rgb(0,200,83)`) 跌红 (`rgb(234,57,67)`)

---

## 3.5 K 线图渲染（核心）

### CandlestickChart 模块

K 线图是 UI 层最核心的组件，使用 egui 底层 `Shape` API 自定义绘制。

```rust
// src/ui/chart.rs
use egui::{pos2, vec2, Color32, Pos2, Rect, Response, Sense, Stroke, Ui};
use egui::epaint::{RectShape, PathShape, CircleShape};

pub struct CandlestickChart {
    /// 可视区域起始时间索引（平移偏移量）
    scroll_offset: usize,
    /// 每根蜡烛的像素宽度
    candle_width: f32,
    /// 蜡烛间距比例
    gap_ratio: f32,
}

/// 单根蜡烛的绘制参数
struct CandleGeometry {
    body_rect: Rect,      // 实体矩形 (open-close)
    wick_top: (Pos2, Pos2), // 上影线 (high → body_top)
    wick_bot: (Pos2, Pos2), // 下影线 (low → body_bottom)
    color: Color32,
}

impl CandlestickChart {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            candle_width: 8.0,
            gap_ratio: 0.3,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        let klines = &state.current_klines;
        if klines.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("暂无 K 线数据");
            });
            return;
        }

        // 计算可视范围
        let available_count = (ui.available_width() / self.candle_width) as usize;
        let end = klines.len().saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(available_count);
        let visible = &klines[start..end];

        // 计算价格范围
        let (price_min, price_max) = self.price_range(visible);
        let price_padding = (price_max - price_min) * 0.05;
        let price_min = price_min - price_padding;
        let price_max = price_max + price_padding;

        // 分配绘制区域（留出坐标轴空间）
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size(),
            Sense::click_and_drag(),
        );

        // 坐标变换闭包
        let price_to_y = |price: f64| -> f32 {
            let ratio = (price - price_min) / (price_max - price_min);
            rect.bottom() - (ratio as f32 * rect.height())
        };
        let index_to_x = |i: usize| -> f32 {
            rect.left() + (i as f32 + 0.5) * self.candle_width
        };

        // 绘制背景网格
        self.draw_grid(ui, rect, price_min, price_max, &price_to_y);

        // 绘制每根蜡烛
        let painter = ui.painter_at(rect);
        for (i, kline) in visible.iter().enumerate() {
            let geo = self.candle_geometry(kline, i, &price_to_y, &index_to_x);
            self.draw_candle(&painter, &geo);
        }

        // 绘制坐标轴
        self.draw_price_axis(ui, rect, price_min, price_max, &price_to_y);
        self.draw_time_axis(ui, rect, visible, &index_to_x);

        // 处理交互：缩放 + 平移
        self.handle_zoom(&response, state);
        self.handle_pan(&response);
    }

    /// 计算单根蜡烛的几何参数
    fn candle_geometry(
        &self,
        kline: &Kline,
        index: usize,
        price_to_y: &impl Fn(f64) -> f32,
        index_to_x: &impl Fn(usize) -> f32,
    ) -> CandleGeometry {
        let x = index_to_x(index);
        let half_w = self.candle_width * (1.0 - self.gap_ratio) / 2.0;
        let is_bullish = kline.close >= kline.open;

        let body_top = price_to_y(kline.open.max(kline.close));
        let body_bot = price_to_y(kline.open.min(kline.close));
        let body_rect = Rect::from_min_max(
            pos2(x - half_w, body_top),
            pos2(x + half_w, body_bot.max(body_top + 1.0)), // 最小 1px 高度
        );

        let color = if is_bullish {
            Color32::from_rgb(0, 200, 83)     // 涨：绿色
        } else {
            Color32::from_rgb(234, 57, 67)     // 跌：红色
        };

        CandleGeometry {
            body_rect,
            wick_top: (pos2(x, price_to_y(kline.high)), pos2(x, body_top)),
            wick_bot: (pos2(x, body_bot), pos2(x, price_to_y(kline.low))),
            color,
        }
    }

    /// 绘制单根蜡烛
    fn draw_candle(&self, painter: &egui::Painter, geo: &CandleGeometry) {
        // 影线（上下各一条细线）
        let wick_stroke = Stroke::new(1.0, geo.color);
        painter.line_segment(geo.wick_top, wick_stroke);
        painter.line_segment(geo.wick_bot, wick_stroke);

        // 实体矩形
        painter.add(RectShape::filled(geo.body_rect, 0.0, geo.color));
    }

    /// 鼠标滚轮缩放
    fn handle_zoom(&mut self, response: &Response, _state: &mut AppState) {
        if response.hovered() {
            let scroll = _state.ctx_input_scroll(); // 从 ctx 获取滚轮增量
            if scroll != 0.0 {
                self.candle_width = (self.candle_width + scroll * 0.5).clamp(2.0, 30.0);
            }
        }
    }

    /// 鼠标拖拽平移
    fn handle_pan(&mut self, response: &Response) {
        if response.dragged_by(egui::PointerButton::Primary) {
            let dx = response.drag_delta().x;
            let candles_delta = (dx / self.candle_width).round() as i32;
            if candles_delta != 0 {
                self.scroll_offset = self.scroll_offset.saturating_add_signed(-candles_delta);
            }
        }
    }

    /// 计算可视区域的价格范围
    fn price_range(&self, klines: &[Kline]) -> (f64, f64) {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for k in klines {
            min = min.min(k.low);
            max = max.max(k.high);
        }
        (min, max)
    }

    /// 绘制背景网格线
    fn draw_grid(&self, ui: &mut Ui, rect: Rect, price_min: f64, price_max: f64,
                 price_to_y: &impl Fn(f64) -> f32) { /* ... */ }

    /// 绘制价格轴（右侧）
    fn draw_price_axis(&self, ui: &mut Ui, rect: Rect, price_min: f64, price_max: f64,
                       price_to_y: &impl Fn(f64) -> f32) { /* ... */ }

    /// 绘制时间轴（底部）
    fn draw_time_axis(&self, ui: &mut Ui, rect: Rect, klines: &[Kline],
                      index_to_x: &impl Fn(usize) -> f32) { /* ... */ }
}
```

### 坐标变换算法说明

```
价格 → Y 像素：
  ratio = (price - price_min) / (price_max - price_min)
  y = rect.bottom - ratio * rect.height

  注意：屏幕坐标系 Y 轴向下，价格越高 Y 越小

索引 → X 像素：
  x = rect.left + (index + 0.5) * candle_width

  +0.5 使蜡烛居中在格子内
```

### 设计要点

- **Shape API 选择**：`RectShape` 绘制蜡烛实体，`line_segment` 绘制影线，性能优于 `PathShape`
- **颜色规范**：涨绿 `rgb(0,200,83)`、跌红 `rgb(234,57,67)`，与侧边栏涨跌幅一致
- **缩放范围**：`candle_width` 限制在 `[2.0, 30.0]`，防止过小或过大
- **性能考虑**：只绘制可视区域内的蜡烛（`start..end` 切片），避免全量遍历
- **最小高度**：实体矩形高度至少 1px (`body_bot.max(body_top + 1.0)`)，防止十字线不可见

---

## 3.6 指标面板

### IndicatorPanel 结构体

```rust
// src/ui/indicator_panel.rs
use egui::{Ui, Color32};
use egui_plot::{Plot, Line, PlotPoints};

/// 当前选中的指标 Tab
#[derive(Clone, Copy, PartialEq)]
pub enum ActiveIndicator {
    Sma,
    Ema,
    Rsi,
    Macd,
    Bollinger,
}

pub struct IndicatorPanel {
    pub active: ActiveIndicator,
    /// 指标参数（可在 UI 调整）
    pub sma_period: usize,
    pub ema_period: usize,
    pub rsi_period: usize,
    pub macd_fast: usize,
    pub macd_slow: usize,
    pub macd_signal: usize,
    pub bb_period: usize,
    pub bb_std_dev: f64,
}

impl IndicatorPanel {
    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        // Tab 切换栏
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active, ActiveIndicator::Sma, "SMA");
            ui.selectable_value(&mut self.active, ActiveIndicator::Ema, "EMA");
            ui.selectable_value(&mut self.active, ActiveIndicator::Rsi, "RSI");
            ui.selectable_value(&mut self.active, ActiveIndicator::Macd, "MACD");
            ui.selectable_value(&mut self.active, ActiveIndicator::Bollinger, "BB");

            ui.separator();

            // 当前指标参数调整
            match self.active {
                ActiveIndicator::Sma => {
                    ui.add(egui::Slider::new(&mut self.sma_period, 5..=200).text("Period"));
                }
                ActiveIndicator::Ema => {
                    ui.add(egui::Slider::new(&mut self.ema_period, 5..=200).text("Period"));
                }
                ActiveIndicator::Rsi => {
                    ui.add(egui::Slider::new(&mut self.rsi_period, 5..=50).text("Period"));
                }
                ActiveIndicator::Macd => {
                    ui.label("Fast/Slow/Signal:");
                    ui.add(egui::DragValue::new(&mut self.macd_fast).range(2..=50));
                    ui.add(egui::DragValue::new(&mut self.macd_slow).range(2..=100));
                    ui.add(egui::DragValue::new(&mut self.macd_signal).range(2..=50));
                }
                ActiveIndicator::Bollinger => {
                    ui.add(egui::Slider::new(&mut self.bb_period, 5..=50).text("Period"));
                    ui.add(egui::Slider::new(&mut self.bb_std_dev, 1.0..=3.0).text("StdDev"));
                }
            }
        });

        ui.separator();

        // 指标图表区域
        if let Some(values) = &state.indicator_values {
            self.draw_indicator_chart(ui, values);
        }
    }

    /// 使用 egui_plot 绘制指标折线
    fn draw_indicator_chart(&self, ui: &mut Ui, values: &IndicatorValues) {
        match self.active {
            ActiveIndicator::Sma | ActiveIndicator::Ema => {
                // 单线指标：收盘价 + 指标线
                if let Some(ref line_data) = values.sma.as_ref().or(values.ema.as_ref()) {
                    let points: PlotPoints = line_data
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| [i as f64, v])
                        .collect();
                    Plot::new("indicator_plot")
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(points).name("MA").color(Color32::from_rgb(255, 193, 7)));
                        });
                }
            }
            ActiveIndicator::Rsi => {
                // RSI：带超买/超卖参考线
                if let Some(ref rsi_data) = values.rsi {
                    let points: PlotPoints = rsi_data
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| [i as f64, v])
                        .collect();
                    Plot::new("indicator_plot")
                        .include_y(0.0)
                        .include_y(100.0)
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(points).name("RSI").color(Color32::from_rgb(156, 39, 176)));
                            // 超买/超卖水平线
                            plot_ui.hline(egui_plot::HLineImg::new(70.0).name("Overbought"));
                            plot_ui.hline(egui_plot::HLineImg::new(30.0).name("Oversold"));
                        });
                }
            }
            ActiveIndicator::Macd => {
                // MACD：DIF/DEA/柱状图三线
                // ... 类似实现
            }
            ActiveIndicator::Bollinger => {
                // 布林带：upper/middle/lower 三线
                // ... 类似实现
            }
        }
    }
}
```

### 设计要点

- `selectable_value` 实现 Tab 切换，比手动判断更简洁
- 参数通过 `Slider` / `DragValue` 实时调整，修改后自动触发重新计算
- `egui_plot` 的 `Plot::show()` 提供自动坐标轴、缩放、平移
- RSI 指标需绘制 70/30 参考线，使用 `hline` 绘制水平线

---

## 3.7 工具栏与状态栏

### 工具栏

```rust
impl QuantApp {
    fn toolbar_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // 交易对下拉选择
            ui.label("交易对:");
            egui::ComboBox::from_id_salt("symbol_select")
                .selected_text(&self.state.selected_symbol)
                .show_ui(ui, |ui| {
                    for symbol in &self.state.available_symbols {
                        ui.selectable_value(
                            &mut self.state.selected_symbol,
                            symbol.clone(),
                            symbol,
                        );
                    }
                });

            ui.separator();

            // K 线周期切换按钮组
            ui.label("周期:");
            for interval in [Interval::M1, Interval::M5, Interval::M15,
                             Interval::H1, Interval::H4, Interval::D1] {
                let label = interval.as_str();
                let is_selected = self.state.current_interval == interval;
                if ui.selectable_label(is_selected, label).clicked() {
                    self.state.current_interval = interval;
                    self.state.load_kline_data();
                }
            }
        });
    }
}
```

### 状态栏

```rust
impl QuantApp {
    fn status_bar_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // 连接状态指示灯
            let (dot_color, status_text) = if self.state.is_connected {
                (Color32::from_rgb(0, 200, 83), "已连接")
            } else {
                (Color32::from_rgb(234, 57, 67), "未连接")
            };
            ui.add(egui::Label::new(
                egui::RichText::new("●").color(dot_color)
            ));
            ui.label(status_text);

            ui.separator();

            // 最新价格
            if let Some(ticker) = &self.state.latest_ticker {
                ui.label(format!("{}: {:.2}", ticker.symbol, ticker.price));
            }

            ui.separator();

            // 最后更新时间
            if let Some(ts) = self.state.last_update_time {
                let time_str = chrono::DateTime::from_timestamp_millis(ts)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_default();
                ui.label(format!("更新: {}", time_str));
            }

            ui.separator();

            // 数据条数
            ui.label(format!("K 线: {} 条", self.state.current_klines.len()));
        });
    }
}
```

### 设计要点

- `ComboBox` 用于交易对选择，`selectable_label` 用于周期切换按钮组
- 状态栏连接状态用颜色圆点指示（绿=已连接，红=未连接）
- 时间使用 `chrono` 格式化，毫秒时间戳→`HH:MM:SS`

---

## 3.8 应用状态管理

### AppState 结构体

```rust
// src/ui/app.rs
use std::sync::Arc;
use tokio::sync::Mutex;

/// 集中管理所有 UI 状态
pub struct AppState {
    // === 交易对与周期 ===
    pub selected_symbol: String,
    pub current_interval: Interval,
    pub available_symbols: Vec<String>,

    // === K 线数据 ===
    pub current_klines: Vec<Kline>,

    // === 指标数据 ===
    pub indicator_values: Option<IndicatorValues>,

    // === 实时数据（Phase 4 填充）===
    pub latest_ticker: Option<Ticker>,
    pub is_connected: bool,
    pub last_update_time: Option<i64>,

    // === egui 输入（用于获取滚轮等事件）===
    #[serde(skip)]
    raw_input: egui::RawInput,

    // === 数据层引用（Phase 4 添加 channel receiver）===
    // ticker_rx: Option<mpsc::UnboundedReceiver<Ticker>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_symbol: "BTCUSDT".to_string(),
            current_interval: Interval::M15,
            available_symbols: vec![
                "BTCUSDT".into(), "ETHUSDT".into(), "BNBUSDT".into(),
                "SOLUSDT".into(), "XRPUSDT".into(), "DOGEUSDT".into(),
            ],
            current_klines: Vec::new(),
            indicator_values: None,
            latest_ticker: None,
            is_connected: false,
            last_update_time: None,
            raw_input: egui::RawInput::default(),
        }
    }

    /// 加载当前交易对和周期的 K 线数据
    pub fn load_kline_data(&mut self) {
        // Phase 3: 同步从 KlineStore 加载
        // Phase 4: 异步加载，通过 channel 接收
        self.current_klines = Vec::new(); // placeholder
        self.recompute_indicators();
    }

    /// 重新计算指标
    pub fn recompute_indicators(&mut self) {
        if self.current_klines.is_empty() {
            self.indicator_values = None;
            return;
        }
        let closes: Vec<f64> = self.current_klines.iter().map(|k| k.close).collect();
        // 调用 indicators 层计算各指标...
        self.indicator_values = Some(IndicatorValues {
            closes,
            sma: None,   // 实际计算填充
            ema: None,
            rsi: None,
            macd: None,
            bollinger: None,
        });
    }
}
```

### 状态更新流程

```
用户操作（切换交易对/周期）
      │
      ▼
AppState.load_kline_data()
      │
      ▼
KlineStore.get(symbol, interval) ──▶ Vec<Kline>
      │
      ▼
AppState.recompute_indicators()
      │
      ▼
Indicator.compute(closes) ──▶ IndicatorValues
      │
      ▼
egui 下一帧 update() 自动重绘
```

### 设计要点

- `AppState` 集中管理所有 UI 状态，避免状态分散在多个结构体中
- `load_kline_data()` 触发数据加载 → 指标重算 → 自动重绘（即时模式无需手动刷新）
- Phase 4 添加 `ticker_rx` channel receiver，在 `update()` 中 `try_recv()` 消费实时数据
- `available_symbols` 初始硬编码，后续可从 `GET /api/v3/exchangeInfo` 动态获取

---

## 3.9 设计要点

### egui 即时模式的心智模型

- **每帧重绘**：`update()` 每秒调用 30-60 次，所有 UI 元素在每次调用中重新构建
- **无 UI 树**：不像 React/HTML 有持久 DOM，egui 的 widget 是临时创建的
- **状态持久化**：widget 状态通过 `Id` 系统自动持久化（如 `ScrollArea` 的滚动位置）
- **响应式更新**：无需手动触发刷新，修改 `AppState` 后下一帧自动反映

### 性能考虑

| 场景 | 优化策略 |
|------|---------|
| 大数据集 K 线渲染 | 只绘制可视范围内的蜡烛（viewport culling） |
| 指标计算 | 数据量大时在后台线程计算，避免阻塞 UI 线程 |
| 实时数据更新 | `try_recv()` 非阻塞消费，避免 UI 卡顿 |
| 字体加载 | 启动时一次性加载，使用 `include_bytes!` 编译嵌入 |

### ctx.request_repaint() 的使用场景

- 收到 WebSocket 实时数据时，调用 `ctx.request_repaint()` 唤醒 UI 线程重绘
- 异步数据加载完成时，通知 UI 刷新
- 默认情况下 egui 在用户交互（鼠标移动/键盘）时自动重绘，静态时无需重绘
- `request_repaint()` 打破"空闲不重绘"行为，确保实时数据及时显示
