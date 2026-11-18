use std::sync::Arc;

use crate::bridge::{self, DataBridge};
use crate::ui::theme::{AppThemePalette, theme_palette};
use gpui_kit::component::ActiveTheme as _;

use chrono::Utc;
use gpui_kit::component::table::{Column, ColumnSort, DataTable, TableDelegate, TableState};
use gpui_kit::{
    base::{h_flex, v_flex},
    *,
};

use stock_data::{PollingHandle, StockBoard};

struct MyTableDelegate {
    data: Vec<StockBoard>,
    columns: Vec<Column>,
}

impl MyTableDelegate {
    fn new(data: Vec<StockBoard>) -> Self {
        Self {
            data,
            columns: vec![
                Column::new("index", "排名").width(50.0),
                Column::new("name", "名称"),
                Column::new("change_percent", "涨跌幅").width(80.0),
                Column::new("leading_stock", "领涨股").width(80.0),
                Column::new("rise_fall", "涨/跌").width(80.0),
                Column::new("hands", "换手").width(50.0),
            ],
        }
    }
}
impl TableDelegate for MyTableDelegate {
    fn columns_count(&self, cx: &App) -> usize {
        self.columns.len()
    }
    fn rows_count(&self, cx: &App) -> usize {
        self.data.len()
    }
    fn column(&self, col_ix: usize, cx: &App) -> Column {
        self.columns[col_ix].clone()
    }
    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut Window,
        cx: &mut Context<component::table::TableState<Self>>,
    ) -> impl IntoElement {
        let row = &self.data[row_ix];
        let col = &self.columns[col_ix];
        let theme = AppThemePalette::for_appearance(cx.theme().is_dark());
        match col.key.as_ref() {
            "index" => format!("{}", row_ix + 1).into_any_element(),
            "name" => row.name.clone().into_any_element(),
            "change_percent" => div()
                .text_color(
                    Some(row.change_percent)
                        .map(|n| {
                            if n > 0.0 {
                                return theme.color_rise;
                            } else if n < 0.0 {
                                return theme.color_fall;
                            } else {
                                return theme.text_tertiary;
                            }
                        })
                        .unwrap(),
                )
                .child(format!("{:+.2}", row.change_percent))
                .into_any_element(),
            "leading_stock" => v_flex()
                .items_center()
                .children(vec![
                    row.leading_stock.clone().into_any_element(),
                    div()
                        .text_color(
                            Some(row.leading_stock_change_percent)
                                .map(|n| {
                                    if n > 0.0 {
                                        return theme.color_rise;
                                    } else if n < 0.0 {
                                        return theme.color_fall;
                                    } else {
                                        return theme.text_tertiary;
                                    }
                                })
                                .unwrap(),
                        )
                        .child(format!("{:+.2}", row.leading_stock_change_percent))
                        .into_any_element(),
                ])
                .into_any_element(),
            "rise_fall" => row.rise_count.to_string().into_any_element(),
            "hands" => row.rise_count.to_string().into_any_element(),
            _ => "".to_string().into_any_element(),
        }
    }
}
pub struct RanklistPanel {
    _bridge: Arc<DataBridge>,
    // 接口轮询句柄
    polling_handle: Vec<PollingHandle>,
    industry_board: Vec<StockBoard>,
    concept_board: Vec<StockBoard>,
}
impl RanklistPanel {
    pub fn new(bridge: Arc<DataBridge>, cx: &mut Context<Self>) -> Self {
        let (tx, rx) = async_channel::unbounded::<Vec<StockBoard>>();
        let handel1 = bridge.start_polling_industry_board(tx);

        cx.spawn(async move |this, cx| {
            while let Ok(data) = rx.recv().await {
                println!("{:?}", data);
                let _ = this.update(cx, |s, cx| {
                    s.industry_board = data;
                    cx.notify();
                });
            }
        })
        .detach();

        let (tx2, rx2) = async_channel::unbounded::<Vec<StockBoard>>();
        let handel2 = bridge.start_polling_concept_board(tx2);

        cx.spawn(async move |this, cx| {
            while let Ok(data) = rx2.recv().await {
                println!("{:?}", data);
                let _ = this.update(cx, |s, cx| {
                    s.concept_board = data;
                    cx.notify();
                });
            }
        })
        .detach();
        Self {
            _bridge: bridge.clone(),
            industry_board: vec![],
            concept_board: vec![],
            polling_handle: vec![handel1, handel2],
        }
    }
    fn render_industry_table(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let data = self.industry_board.clone();
        println!("{:?}", data);
        let delegate = MyTableDelegate::new(data);
        let state = cx.new(|cx| TableState::new(delegate, window, cx));
        state
    }
    fn render_concept_table(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let data = self.concept_board.clone();
        let delegate = MyTableDelegate::new(data);
        let state = cx.new(|cx| TableState::new(delegate, window, cx));
        state
    }
}
impl Render for RanklistPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().h_full().grid().grid_cols(2).gap_3().children(vec![
            div()
                .h_full()
                .border_1()
                .child(self.render_industry_table(window, cx)),
            div()
                .h_full()
                .border_1()
                .child(self.render_concept_table(window, cx)),
        ])
    }
}
