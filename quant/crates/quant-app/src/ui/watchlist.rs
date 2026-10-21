use std::default;

use gpui_kit::base::input::InputEvent;
use gpui_kit::base::{Selectable, h_flex, v_flex};
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dock::{BasePanel, Panel, PanelEvent};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::{FocusHandle, Focusable, IntoElement};
use rand::Rng;

use crate::ui::app::AppState;

pub struct WatchlistPanel {
    pub app_state: Entity<AppState>,
    pub focus_handle: FocusHandle,
    pub filter_input: Entity<InputState>, // 搜索框
    pub items: Vec<WatchlistItem>,
    sub: Vec<Subscription>,
}

pub struct WatchlistItem {
    pub symbol: String,
    pub price: f64,
    pub change_24h: f64,
}
impl BasePanel for WatchlistPanel {
    fn panel_name(&self) -> &'static str {
        "交易对"
    }
    fn zoomable(&self, _cx: &App) -> bool {
        false
    }
    fn closable(&self, _cx: &App) -> bool {
        false
    }
}
impl Panel for WatchlistPanel {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "品种列表"
    }
}
impl Focusable for WatchlistPanel {
    fn focus_handle(&self, cx: &gpui_kit::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for WatchlistPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(
                div()
                    .p_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(Input::new(&self.filter_input)),
            )
            .child(self.render_items(cx))
    }
}
impl EventEmitter<PanelEvent> for WatchlistPanel {}

impl WatchlistPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, appstate: Entity<AppState>) -> Self {
        let mut mock_items = Vec::new();
        let mut rng = rand::thread_rng();
        let mut sub: Vec<Subscription> = Vec::with_capacity(2);
        for i in 0..10 {
            mock_items.push(WatchlistItem {
                symbol: format!("BTC{}_usdt", i),
                price: rng.gen_range(80.0..100.0),
                change_24h: rng.gen_range(-10.0..10.0),
            });
        }
        sub.push(cx.observe(&appstate, |_, _state, cx| {
            cx.notify();
        }));
        let filter_input = cx.new(|cx| InputState::new(window, cx));

        sub.push(cx.subscribe_in(&filter_input, window, Self::handle_input_change));

        Self {
            focus_handle: cx.focus_handle(),
            filter_input,
            items: mock_items,
            app_state: appstate,
            sub: sub,
        }
    }
    fn handle_input_change(
        this: &mut Self,
        _emitter: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = this.filter_input.read(cx).value();
                println!("{:?}", text);
                cx.notify();
            }
            _ => (),
        }
    }
    fn render_items(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // let state = self.
        let selected = self.app_state.read(cx).selected_symbol.clone();
        v_flex()
            .overflow_y_scrollbar()
            .children(self.filtered_items(cx).into_iter().map(|item| {
                let color = if item.change_24h >= 0.0 {
                    cx.theme().success
                } else {
                    cx.theme().danger
                };
                let symbol = item.symbol.clone();
                Button::new(item.symbol.clone())
                    .ghost()
                    .selected(item.symbol == selected)
                    .child(
                        h_flex()
                            .justify_between()
                            .px_2()
                            .py_1()
                            .gap_2()
                            .child(item.symbol.clone())
                            .child(format!("{:.2}", item.price))
                            .child(
                                div()
                                    .text_color(color)
                                    .child(format!("{:.2}", item.change_24h)),
                            ),
                    )
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.select_symbol(&symbol, _window, cx);
                    }))
            }))
    }
    fn filtered_items(&self, cx: &mut Context<Self>) -> Vec<&WatchlistItem> {
        let filter_input_state = self.filter_input.read(cx);
        let filter = filter_input_state.value().to_lowercase();
        if filter.is_empty() {
            self.items.iter().collect()
        } else {
            self.items
                .iter()
                .filter(|item| item.symbol.to_lowercase().contains(&filter))
                .collect()
        }
    }

    fn select_symbol(&mut self, symbol: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.app_state.update(cx, |p, cx| {
            p.selected_symbol = symbol.to_string();
            cx.notify();
        });
        println!("select_symbol,{}", symbol);
    }
}
