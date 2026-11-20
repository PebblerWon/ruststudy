use std::sync::Arc;

use gpui_kit::*;

use crate::{
    bridge::DataBridge,
    ui::app::{DetailTarget, StockAppState},
};

#[derive(Clone, PartialEq)]
pub struct DetailPanel {
    target: DetailTarget,
}

impl Render for DetailPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(format!("detail of : {}", self.target.name().clone()))
    }
}

impl DetailPanel {
    pub fn new(
        bridge: Arc<DataBridge>,
        target: DetailTarget,
        app_state: Entity<StockAppState>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self { target }
    }
    pub fn stop_polling(&self) {}

    pub fn target(&self) -> &DetailTarget {
        &self.target
    }
}
