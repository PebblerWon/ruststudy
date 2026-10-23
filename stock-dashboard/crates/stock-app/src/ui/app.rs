use gpui_kit::*;
pub struct StockAppState {}

impl StockAppState {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct StockApp {}
impl StockApp {
    pub fn new() -> Self {
        Self {}
    }
}
impl Render for StockApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Stock App")
    }
}
