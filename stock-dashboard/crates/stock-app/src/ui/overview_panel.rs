use std::sync::Arc;

use crate::{
    bridge::DataBridge,
    ui::{
        app::StockAppState,
        theme::{AppThemePalette, theme_palette},
    },
};
use gpui_kit::{
    base::{StyledExt, h_flex, v_flex},
    *,
};
use stock_data::{IndexQuote, MarketSnapshot, PollingHandle};
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
    app_state: Entity<StockAppState>,
    market_snapshot: Option<MarketSnapshot>,
}
impl OverviewPanel {
    pub fn new(
        bridge: Arc<DataBridge>,
        cx: &mut Context<Self>,
        app_state: Entity<StockAppState>,
    ) -> Self {
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
        cx.observe(&app_state, Self::handle_app_state_change)
            .detach();
        let snapshots = Vec::with_capacity(6);
        let theme = theme_palette(cx);
        Self {
            _bridge: bridge,
            snapshots,
            polling_handle,
            theme,
            app_state,
            market_snapshot: None,
        }
    }
    fn handle_app_state_change(&mut self, _state: Entity<StockAppState>, cx: &mut Context<Self>) {
        let all_a_codes = &_state.read(cx).all_a_codes;
        let bridge = self._bridge.clone();
        if let Some(codes) = all_a_codes {
            if codes.len() > 0 {
                let (tx, rx) = async_channel::unbounded::<stock_data::MarketSnapshot>();
                let _ = bridge.start_polling_market_snapshot(tx);
                cx.spawn(async move |this, cx| {
                    while let Ok(market_snapshot) = rx.recv().await {
                        let _ = this.update(cx, |s, cx| {
                            // s.last_refresh = Some(chrono::Local::now());
                            s.market_snapshot = Some(market_snapshot);
                            cx.notify();
                        });
                    }
                })
                .detach();
            }
        }
    }
    fn update_snapshots(&mut self, new_snapshots: Vec<IndexQuote>) {
        // println!("receive index_quotes: {:?}", new_snapshots);
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
        // .hover(|style| {
        //     style
        //         .shadow_md()
        //         .border_color(theme.color_rise)
        //         .cursor_pointer()
        // })
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
        let market_snapshot = &self.market_snapshot;

        div()
            .grid()
            .grid_cols(3)
            .grid_rows(2)
            .gap_3()
            .w_full()
            .justify_between()
            .children(vec![
                self.card()
                    .h_32()
                    .child(
                        div()
                            .mb_2()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .font_bold()
                            .child(format!("市场涨跌")),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .text_2xl()
                            .text_color(theme.color_rise)
                            .children(vec![
                                div().text_color(theme.color_rise).child(
                                    market_snapshot
                                        .as_ref()
                                        .map_or("--".to_string(), |s| s.up_count.to_string()),
                                ),
                                div().text_color(theme.text_tertiary).child(" / "),
                                div().text_color(theme.color_fall).child(
                                    market_snapshot
                                        .as_ref()
                                        .map_or("--".to_string(), |s| s.down_count.to_string()),
                                ),
                            ]),
                    )
                    .child(
                        h_flex()
                            .mt_auto()
                            .justify_between()
                            .text_xs()
                            .text_color(theme.text_tertiary)
                            .children(vec![
                                div().child("上涨 / 下跌"),
                                div().child(format!(
                                    "{}平",
                                    market_snapshot
                                        .as_ref()
                                        .map_or("--".to_string(), |s| s.flat_count.to_string())
                                )),
                            ]),
                    ),
                self.card()
                    .h_32()
                    .child(
                        div()
                            .mb_2()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .font_bold()
                            .child(format!("涨跌停")),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .text_2xl()
                            .text_color(theme.color_rise)
                            .children(vec![
                                div().text_color(theme.color_rise).child(
                                    market_snapshot
                                        .as_ref()
                                        .map_or("--".to_string(), |s| s.limit_up.to_string()),
                                ),
                                div().text_color(theme.text_tertiary).child("/"),
                                div().text_color(theme.color_fall).child(
                                    market_snapshot
                                        .as_ref()
                                        .map_or("--".to_string(), |s| s.limit_down.to_string()),
                                ),
                            ]),
                    )
                    .child(
                        h_flex()
                            .mt_auto()
                            .justify_between()
                            .text_xs()
                            .text_color(theme.text_tertiary)
                            .children(vec![div().child("涨停 / 跌停")]),
                    ),
                self.card()
                    .h_32()
                    .child(
                        div()
                            .mb_2()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .font_bold()
                            .child(format!("全市场成交额")),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .text_2xl()
                            .text_color(theme.text_secondary)
                            .child(market_snapshot.as_ref().map_or("--".to_string(), |s| {
                                // 注意：不能写成 format!("{:.2}亿", s.total_amount.to_string())，
                                // {:.2} 对字符串是截取前 2 字符，会把 10456.78 显示成 "10亿"
                                format!("{:.2}亿", s.total_amount)
                            })),
                    )
                    .child(
                        h_flex()
                            .mt_auto()
                            .justify_between()
                            .text_xs()
                            .text_color(theme.text_tertiary)
                            .child(format!("A 股实时成交额快照")),
                    ),
                self.card()
                    .h_32()
                    .child(
                        div()
                            .mb_2()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .font_bold()
                            .child(format!("北向资金")),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .text_2xl()
                            .text_color(theme.text_secondary)
                            .child(format!("0.54亿")),
                    )
                    .child(
                        h_flex()
                            .mt_auto()
                            .justify_between()
                            .text_xs()
                            .text_color(theme.text_tertiary)
                            .children(vec![
                                div().child(format!("上涨 971 / 下跌 639")),
                                div().child(format!("沪股通")),
                            ]),
                    ),
                self.card()
                    .h_32()
                    .child(
                        div()
                            .mb_2()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .font_bold()
                            .child(format!("大盘主力")),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .text_2xl()
                            .text_color(theme.text_secondary)
                            .child(format!("-396.66亿")),
                    )
                    .child(
                        h_flex()
                            .mt_auto()
                            .justify_between()
                            .text_xs()
                            .text_color(theme.text_tertiary)
                            .children(vec![
                                div().child(format!("占比 -2.37%")),
                                div().child(format!("2026-10-30")),
                            ]),
                    ),
                self.card()
                    .h_32()
                    .child(
                        div()
                            .mb_2()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .font_bold()
                            .child(format!("最强板块")),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .text_2xl()
                            .text_color(theme.text_secondary)
                            .child(format!("视频媒体")),
                    )
                    .child(
                        h_flex()
                            .mt_auto()
                            .justify_between()
                            .text_xs()
                            .text_color(theme.text_tertiary)
                            .children(vec![
                                div().text_color(theme.color_rise).child(format!("-2.37%")),
                                div().child(format!("行业强度")),
                            ]),
                    ),
            ])
    }
}
impl Render for OverviewPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_full()
            .v_flex()
            .gap_3()
            .child(self.render_indicator_cards(window, cx))
            .child(self.render_market_content(cx))
    }
}
