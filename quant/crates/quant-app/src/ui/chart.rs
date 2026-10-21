use gpui_kit::component::ActiveTheme;
use gpui_kit::component::chart::CandlestickChart;
use gpui_kit::component::dock::{BasePanel, Panel, PanelEvent};
use gpui_kit::*;
use gpui_kit::{FocusHandle, Focusable, IntoElement};
use quant_data::common::models::Kline;
use rand::Rng;

#[derive(Clone)]
pub struct CandleData {
    pub time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn to_candles(klines: &[Kline]) -> Vec<CandleData> {
    klines
        .iter()
        .map(|k| CandleData {
            time: chrono::DateTime::from_timestamp_millis(k.open_time)
                .map(|dt| dt.format("%m-%d %H:%M").to_string())
                .unwrap_or_default(),
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
        })
        .collect()
}

pub struct ChartPanel {
    pub focus_handle: FocusHandle,
    pub klines: Vec<Kline>,
}
impl BasePanel for ChartPanel {
    fn panel_name(&self) -> &'static str {
        "k线图"
    }
    fn zoomable(&self, _cx: &App) -> bool {
        false
    }
    fn closable(&self, _cx: &App) -> bool {
        false
    }
}
impl Panel for ChartPanel {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "chart"
    }
}
impl EventEmitter<PanelEvent> for ChartPanel {}

impl Focusable for ChartPanel {
    fn focus_handle(&self, cx: &gpui_kit::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for ChartPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let candles = to_candles(&self.klines);
        if candles.is_empty() {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child("暂无K线数据")
                .into_any_element();
        }
        CandlestickChart::new(candles)
            .x(|d| d.time.clone())
            .open(|d| d.open)
            .high(|d| d.high)
            .low(|d| d.low)
            .close(|d| d.close)
            .body_width_ratio(0.4) // 蜡烛实体宽度占格子比例（默认 0.8）
            .bullish(cx.theme().success) // 阳线：绿涨（加密货币惯例）
            .bearish(cx.theme().danger) // 阴线：红跌
            .into_any_element()
    }
}
impl ChartPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut c: Vec<Kline> = Vec::with_capacity(10);
        let mut rng = rand::thread_rng();

        let start_timestamp = chrono::Utc::now().timestamp_millis();
        let interval = 1000 * 3600 * 24;
        for i in 0..50 {
            let open = rng.gen_range(50.0..100.0);
            let close = rng.gen_range(50.0..100.0);
            let high = open + rng.gen_range(0.0..10.0);
            let low = close - rng.gen_range(0.0..10.0);

            c.push(Kline {
                open_time: start_timestamp + interval * i,
                open,
                high,
                low,
                close_time: start_timestamp + interval * i,
                close,
                volume: 100.0,
                quote_volume: 100.0,
                trades_count: 100,
                is_closed: true,
            });
        }
        Self {
            klines: c,
            focus_handle: cx.focus_handle(),
        }
    }
}
