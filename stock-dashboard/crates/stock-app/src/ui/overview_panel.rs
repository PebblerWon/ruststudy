use crate::{
    ui::theme::{AppThemePalette, theme_palette},
    utils::format::format_wan_yi,
};
use chrono::Utc;
use gpui_kit::{
    base::{h_flex, v_flex},
    component::ActiveTheme,
    *,
};
pub struct IndexSnapshot {
    pub code: String,         // 指数代码 000001.SH
    pub name: String,         // 指数名称：上证指数
    pub last_price: f64,      // 最新点位
    pub change: f64,          // 涨跌额（涨了多少点）
    pub change_pct: f64,      // 涨跌幅（小数，如0.025 = +2.5%）
    pub turnover_amount: f64, // 成交额
    pub timestamp: i64,       // 行情时间戳
}
impl IndexSnapshot {
    pub fn new(
        code: &str,
        name: &str,
        last_price: f64,
        change: f64,
        change_pct: f64,
        turnover_amount: f64,
    ) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            last_price,
            change,
            change_pct,
            turnover_amount,
            timestamp: 0,
        }
    }
}
pub struct OverviewPanel {
    snapshots: Vec<IndexSnapshot>,
}
impl OverviewPanel {
    pub fn new() -> Self {
        let now = Utc::now().timestamp_millis();
        let snapshots = vec![
            IndexSnapshot {
                code: "000001.SH".to_string(),
                name: "上证指数".to_string(),
                last_price: 3328.74,
                change: 18.32,
                change_pct: 0.0055,
                turnover_amount: 2689.42,
                timestamp: now,
            },
            IndexSnapshot {
                code: "399001.SZ".to_string(),
                name: "深证成指".to_string(),
                last_price: 10756.21,
                change: 76.54,
                change_pct: 0.0072,
                turnover_amount: 3142.77,
                timestamp: now,
            },
            IndexSnapshot {
                code: "399006.SZ".to_string(),
                name: "创业板指".to_string(),
                last_price: 2142.68,
                change: -9.73,
                change_pct: -0.0045,
                turnover_amount: 1256.31,
                timestamp: now,
            },
            IndexSnapshot {
                code: "000688.SH".to_string(),
                name: "科创50".to_string(),
                last_price: 967.35,
                change: 14.12,
                change_pct: 0.0148,
                turnover_amount: 487.95,
                timestamp: now,
            },
            IndexSnapshot {
                code: "000906.SH".to_string(),
                name: "中证400".to_string(),
                last_price: 4832.17,
                change: 21.95,
                change_pct: 0.0046,
                turnover_amount: 945.62,
                timestamp: now,
            },
            IndexSnapshot {
                code: "000016.SH".to_string(),
                name: "上证50".to_string(),
                last_price: 2741.89,
                change: 12.06,
                change_pct: 0.0044,
                turnover_amount: 876.28,
                timestamp: now,
            },
        ];
        Self { snapshots }
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
                .map(|snap| self.render_snapshot_card(snap, &theme)),
        )
    }

    fn render_snapshot_card(
        &self,
        snap: &IndexSnapshot,
        theme: &AppThemePalette,
    ) -> impl IntoElement {
        let color = match snap.change_pct >= 0.0 {
            true => theme.color_rise,
            false => theme.color_fall,
        };
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
            .child(div().text_sm().text_color(color).child(format!(
                "{:+.2}%   {}",
                snap.change_pct * 100.0,
                snap.change
            )))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_tertiary)
                    .child(format!("成交 {}", format_wan_yi(snap.turnover_amount))),
            )
    }
}
impl Render for OverviewPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_full()
            .child(self.render_indicator_cards(window, cx))
    }
}
