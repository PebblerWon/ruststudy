use std::sync::Arc;

use crate::{
    bridge::DataBridge,
    ui::theme::{AppThemePalette, theme_palette},
};
use gpui_kit::{
    base::{StyledExt, h_flex, v_flex},
    *,
};
use stock_data::{IndexQuote, PollingHandle};
pub struct IndexSnapshot {
    pub code: String,         // 指数代码 000001.SH
    pub name: String,         // 指数名称：上证指数
    pub last_price: f64,      // 最新点位
    pub change: f64,          // 涨跌额（涨了多少点）
    pub change_pct: f64,      // 涨跌幅（小数，如0.025 = +2.5%）
    pub turnover_amount: f64, // 成交额
}
pub struct OverviewPanel {
    /// 持有 bridge：保住 tokio runtime 生命周期（同 StockApp._bridge）
    _bridge: Arc<DataBridge>,
    snapshots: Vec<IndexSnapshot>,
    polling_handle: PollingHandle,
    theme: AppThemePalette,
}
impl OverviewPanel {
    pub fn new(bridge: Arc<DataBridge>, cx: &mut Context<Self>) -> Self {
        let (tx, rx) = async_channel::unbounded::<Vec<IndexQuote>>();
        let polling_handle = bridge.start_polling_index_quotes(tx);
        cx.spawn(async move |this, cx| {
            while let Ok(quotes) = rx.recv().await {
                let _ = this.update(cx, |s, cx| {
                    // s.last_refresh = Some(chrono::Local::now());
                    s.update_snapshots(quotes);
                    cx.notify();
                });
            }
        })
        .detach();
        let snapshots = Vec::with_capacity(6);
        let theme = theme_palette(cx);
        Self {
            _bridge: bridge,
            snapshots,
            polling_handle,
            theme,
        }
    }
    fn update_snapshots(&mut self, new_snapshots: Vec<IndexQuote>) {
        println!("receive index_quotes: {:?}", new_snapshots);
        let new_snapshots: Vec<IndexSnapshot> = new_snapshots
            .into_iter()
            .map(|quote| IndexSnapshot {
                code: quote.code,
                name: quote.name,
                last_price: quote.price,
                change: quote.change,
                change_pct: quote.change_pct,
                turnover_amount: quote.amount,
            })
            .collect();
        self.snapshots = new_snapshots;
    }
    fn render_indicator_cards(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = theme_palette(cx);
        h_flex().gap_3().w_full().justify_between().children(
            self.snapshots
                .iter()
                .map(|snap| self.render_snapshot_card(snap)),
        )
    }

    fn card(&self) -> Div {
        let theme = self.theme;
        v_flex()
            .w_full()
            .p_3()
            .border(px(1.))
            .border_color(theme.border_primary)
            .rounded_xl()
            .hover(|style| {
                style
                    .shadow_md()
                    .border_color(theme.color_rise)
                    .cursor_pointer()
            })
    }
    fn render_snapshot_card(&self, snap: &IndexSnapshot) -> impl IntoElement {
        let color = match snap.change_pct >= 0.0 {
            true => self.theme.color_rise,
            false => self.theme.color_fall,
        };
        let theme = self.theme;
        self.card()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.text_secondary)
                    .child(snap.name.clone()),
            )
            .child(
                div()
                    .text_xl()
                    .text_color(color)
                    .child(snap.last_price.clone().to_string()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(color)
                    .child(format!("{:+.2}%   {}", snap.change_pct, snap.change)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_tertiary)
                    .child(format!("成交 {:.2}亿", snap.turnover_amount)),
            )
    }

    fn render_market_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let color = theme.color_rise;
        h_flex().gap_3().w_full().justify_between().children(vec![
            self.card()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text_secondary)
                        .child(format!("")),
                )
                .child(div().text_xl().text_color(color).child(format!("")))
                .child(div().text_sm().text_color(color).child(format!("")))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_tertiary)
                        .child(format!("")),
                ),
        ])
    }
}
impl Render for OverviewPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_full()
            .v_flex()
            .gap_8()
            .child(self.render_indicator_cards(window, cx))
    }
}
