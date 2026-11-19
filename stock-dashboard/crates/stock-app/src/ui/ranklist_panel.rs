use std::sync::Arc;

use crate::bridge::{self, DataBridge};
use crate::ui::theme::{AppThemePalette, theme_palette};
use gpui_kit::component::{ActiveTheme as _, Sizable as _};

use chrono::Utc;
use gpui_kit::component::table::{Column, ColumnSort, DataTable, TableDelegate, TableState};
use gpui_kit::{
    base::{h_flex, v_flex},
    *,
};

use stock_data::{PollingHandle, StockBoard};

/// 单排内容的单元格容器：垂直居中，适配加高的表格行
fn cell(child: impl IntoElement) -> AnyElement {
    div()
        .h_full()
        .flex()
        .items_center()
        .child(child)
        .into_any_element()
}

/// 涨跌幅按正负取色
fn percent_color(v: f64, theme: &AppThemePalette) -> Hsla {
    if v > 0.0 {
        theme.color_rise
    } else if v < 0.0 {
        theme.color_fall
    } else {
        theme.text_tertiary
    }
}

/// 让“名称”列吃掉格子剩余宽度，消除固定列宽带来的右侧空白或水平滚动条。
/// 由外层格子的 on_children_prepainted 在每次绘制时回调：读格子内容宽，
/// 动态改名称列宽并刷新表格列布局；宽度没变时不做任何事，保证收敛。
fn fit_name_column(
    table: Entity<TableState<MyTableDelegate>>,
) -> impl Fn(Vec<Bounds<Pixels>>, &mut Window, &mut App) + 'static {
    move |children, _window, cx| {
        let Some(bounds) = children.first() else {
            return;
        };
        // 固定列合计：排名 50 + 涨跌幅/领涨股/涨跌 各 80
        let fixed_cols_width = px(50.) + px(80.) * 3.;
        let name_width = (bounds.size.width - fixed_cols_width).max(px(120.));
        table.update(cx, |table, cx| {
            let Some(ix) = table
                .delegate()
                .columns
                .iter()
                .position(|c| c.key.as_ref() == "name")
            else {
                return;
            };
            if (table.delegate().columns[ix].width - name_width).abs() < px(0.5) {
                return;
            }
            table.delegate_mut().columns[ix].width = name_width;
            // refresh 从 delegate 重新读取列宽并重建列布局
            table.refresh(cx);
            cx.notify();
        });
    }
}

struct MyTableDelegate {
    data: Vec<StockBoard>,
    columns: Vec<Column>,
}

impl MyTableDelegate {
    fn new(data: Vec<StockBoard>) -> Self {
        Self {
            data,
            // StockBoard 没有换手率字段，“换手”列无从渲染，直接去掉
            columns: vec![
                Column::new("index", "排名").width(50.0),
                // 名称列宽度由 fit_name_column 动态计算（吃掉格子剩余宽度），这里只是初值
                Column::new("name", "名称"),
                Column::new("change_percent", "涨跌幅").width(80.0),
                Column::new("leading_stock", "领涨股").width(80.0),
                Column::new("rise_fall", "涨/跌").width(80.0),
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
            "index" => cell(format!("{}", row_ix + 1)),
            "name" => cell(row.name.clone()),
            "change_percent" => cell(
                div()
                    .text_color(percent_color(row.change_percent, &theme))
                    .child(format!("{:+.2}", row.change_percent)),
            ),
            // 上下两排：领涨股名 + 涨跌幅；行高由 DataTable::with_size 保证
            "leading_stock" => v_flex()
                .justify_center()
                .items_center()
                .child(row.leading_stock.clone())
                .child(
                    div()
                        .text_color(percent_color(row.leading_stock_change_percent, &theme))
                        .child(format!("{:+.2}", row.leading_stock_change_percent)),
                )
                .into_any_element(),
            "rise_fall" => cell(format!("{}/{}", row.rise_count, row.fall_count)),
            _ => cell(""),
        }
    }
}
pub struct RanklistPanel {
    _bridge: Arc<DataBridge>,
    // 接口轮询句柄
    polling_handle: Vec<PollingHandle>,
    // 表格状态实体：滚动位置、选中、排序都存在 TableState 里，
    // 必须持久持有，每次 render 重建会把它们全部清零
    industry_table: Entity<TableState<MyTableDelegate>>,
    concept_table: Entity<TableState<MyTableDelegate>>,
}
impl RanklistPanel {
    pub fn new(bridge: Arc<DataBridge>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 表格状态只创建一次；数据更新走 delegate_mut，滚动位置才不会丢
        let industry_table = cx.new(|cx| TableState::new(MyTableDelegate::new(vec![]), window, cx));
        let concept_table = cx.new(|cx| TableState::new(MyTableDelegate::new(vec![]), window, cx));

        let (tx, rx) = async_channel::unbounded::<Vec<StockBoard>>();
        let handel1 = bridge.start_polling_industry_board(tx);

        cx.spawn(async move |this, cx| {
            while let Ok(data) = rx.recv().await {
                let _ = this.update(cx, |s, cx| {
                    s.industry_table.update(cx, |table, cx| {
                        table.delegate_mut().data = data;
                        cx.notify();
                    });
                });
            }
        })
        .detach();

        let (tx2, rx2) = async_channel::unbounded::<Vec<StockBoard>>();
        let handel2 = bridge.start_polling_concept_board(tx2);

        cx.spawn(async move |this, cx| {
            while let Ok(data) = rx2.recv().await {
                let _ = this.update(cx, |s, cx| {
                    s.concept_table.update(cx, |table, cx| {
                        table.delegate_mut().data = data;
                        cx.notify();
                    });
                });
            }
        })
        .detach();
        Self {
            _bridge: bridge.clone(),
            industry_table,
            concept_table,
            polling_handle: vec![handel1, handel2],
        }
    }
}
impl Render for RanklistPanel {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().h_full().grid().grid_cols(2).gap_3().children(vec![
            div()
                .h_full()
                .border_1()
                .on_children_prepainted(fit_name_column(self.industry_table.clone()))
                .child(
                    DataTable::new(&self.industry_table)
                        .with_size(px(48.))
                        .header_row_height(px(36.)),
                ),
            div()
                .h_full()
                .border_1()
                .on_children_prepainted(fit_name_column(self.concept_table.clone()))
                .child(
                    DataTable::new(&self.concept_table)
                        .with_size(px(48.))
                        .header_row_height(px(36.)),
                ),
        ])
    }
}
