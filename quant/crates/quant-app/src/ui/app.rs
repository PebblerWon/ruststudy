use gpui_kit::base::{StyledExt, h_flex, v_flex};
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants};
use gpui_kit::component::select::{Select, SelectState};
use gpui_kit::*;
use quant_data::common::models::Interval;

pub struct AppState {
    pub selected_symbol: String,
    pub current_interval: Interval,
    pub available_symbols: Vec<String>,
    pub is_connected: bool,
    pub last_update_time: Option<i64>,
}
pub struct QuantApp {
    store: Entity<AppState>,
    _subscription: Subscription,
    // pub watchlist:Entity<WatchlistPanel> ,

    // pub char
}
impl QuantApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let store = cx.new(|_| AppState {
            is_connected: true,
            selected_symbol: "".to_string(),
            current_interval: Interval::M1,
            last_update_time: Some(chrono::Utc::now().timestamp()),
            available_symbols: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        });
        let _subscription = cx.observe(&store, |_, _, cx| cx.notify());
        QuantApp {
            store,
            _subscription,
        }
    }
    fn render_toolbar(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.store.read(cx);

        let symbol_state = cx.new(|cx| SelectState::new(vec!["a", "b", "c"], None, window, cx));

        h_flex()
            .px_3()
            .py_2()
            .gap_3()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().border)
            .child("交易对")
            .child(Select::new(&symbol_state).placeholder("请选择").w_40())
            .child(
                ButtonGroup::new("interval")
                    .child(Button::new(Interval::M1.as_str()).label(Interval::M1.as_str()))
                    .child(Button::new(Interval::M5.as_str()).label(Interval::M5.as_str()))
                    .child(Button::new(Interval::M15.as_str()).label(Interval::M15.as_str()))
                    .child(Button::new(Interval::H1.as_str()).label(Interval::H1.as_str()))
                    .child(Button::new(Interval::H4.as_str()).label(Interval::H4.as_str()))
                    .child(Button::new(Interval::D1.as_str()).label(Interval::D1.as_str())),
            )
    }

    fn render_status_bar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = self.store.read(cx);
        let last_update = state
            .last_update_time
            .and_then(chrono::DateTime::from_timestamp_millis)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_default();
        h_flex()
            .px_3()
            .py_1()
            .gap_4()
            .items_center()
            .border_t_1()
            .border_color(cx.theme().border)
            .child("")
            .child(format!("BTCUSDT: {:.2}", 100.482))
            .child(format!("更新： {}", last_update))
            .child(format!("K 线: {} 条", 12))
    }

    fn render_dock(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.store.read(cx);
        v_flex().size_full().flex_1().child("K线图区域")
    }
}
impl Render for QuantApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(self.render_toolbar(window, cx))
            .child(self.render_dock(window, cx))
    }
}
