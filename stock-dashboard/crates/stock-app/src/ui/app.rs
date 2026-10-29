use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
    SidebarToggleButton,
};
use gpui_kit::component::{Icon, IconName};
use gpui_kit::prelude::FluentBuilder as _;

use gpui_kit::*;

use crate::bridge::DataBridge;
use crate::ui::overview_panel::OverviewPanel;
use crate::ui::ranklist_panel::RanklistPanel;
use stock_data::PollingHandle;

pub struct StockAppState {}

impl StockAppState {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub enum AppPanel {
    Overview(Entity<OverviewPanel>),
    Ranklist(Entity<RanklistPanel>),
}

pub struct StockAppMenuItem {
    label: String,
    key: String,
    /// 关联的面板，任意实现 Render 的 View Entity
    panel: AppPanel,
}
pub struct StockApp {
    collapsed: bool,
    active_key: String,

    // polling_handle: PollingHandle,
    // bridge: DataBridge,
    menus: Vec<StockAppMenuItem>,
}

impl StockApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let overview_panel = cx.new(|cx| OverviewPanel::new());
        let ranklist_panel = cx.new(|cx| RanklistPanel::new());

        let menus = vec![
            StockAppMenuItem {
                label: "总览".into(),
                key: "overview".into(),
                panel: AppPanel::Overview(overview_panel),
            },
            StockAppMenuItem {
                label: "榜单".into(),
                key: "ranklist".into(),
                panel: AppPanel::Ranklist(ranklist_panel),
            },
        ];
        // let bridge = DataBridge::new();
        // let (tx, rx) = async_channel::unbounded::<Vec<Quote>>();

        Self {
            collapsed: false,
            active_key: menus[0].key.clone(),
            menus,
        }
    }

    fn render_main_content(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let icon_collapsed = self.collapsed;

        let content_panel = self
            .menus
            .iter()
            .find(|item| item.key == self.active_key)
            .map(|i| i.panel.clone());

        let content_elm = if let Some(panel) = content_panel {
            match panel {
                AppPanel::Overview(v) => v.clone().into_any_element(),
                AppPanel::Ranklist(v) => v.clone().into_any_element(),
            }
        } else {
            div().child("开发中...").into_any_element()
        };

        v_flex()
            .h_full()
            .flex_1()
            .min_w_0()
            .gap_4()
            .p_4()
            .child(
                h_flex().items_center().gap_3().child(
                    SidebarToggleButton::new()
                        .collapsed(icon_collapsed)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.collapsed = !this.collapsed;
                            cx.notify();
                        })),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_2()
                    .child(content_elm),
            )
    }

    fn render_sidebar(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SidebarGroup<SidebarMenu> {
        SidebarGroup::new("A股看板").child(SidebarMenu::new().children(self.menus.iter().map(
            |menu_item| {
                let key = menu_item.key.clone();
                SidebarMenuItem::new(&menu_item.label)
                    .icon(IconName::LayoutDashboard)
                    .active(self.active_key == key)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.active_key = key.clone();
                        cx.notify();
                    }))
            },
        )))
    }
}
impl Render for StockApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let icon_collapsed = self.collapsed;
        h_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                Sidebar::new("sidebar")
                    .collapsible(true)
                    .collapsed(icon_collapsed)
                    .w(px(240.))
                    .header(
                        SidebarHeader::new()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size_8()
                                    .flex_shrink_0()
                                    .rounded(cx.theme().radius)
                                    .bg(cx.theme().sidebar_primary)
                                    .text_color(cx.theme().sidebar_primary_foreground)
                                    .when(icon_collapsed, |this| {
                                        this.size_4()
                                            .bg(cx.theme().transparent)
                                            .text_color(cx.theme().foreground)
                                    })
                                    .child(Icon::new(IconName::GalleryVerticalEnd)),
                            )
                            .when(!icon_collapsed, |this| {
                                this.child(v_flex().flex_1().overflow_hidden().child("Rust Stock"))
                            }),
                    )
                    .child(self.render_sidebar(window, cx))
                    .footer(
                        SidebarFooter::new().child(
                            div()
                                .flex()
                                .items_center()
                                .p_2()
                                .gap_x_2()
                                .rounded(cx.theme().radius)
                                .text_sm()
                                .child(Icon::new(IconName::Settings))
                                .child("设置"),
                        ),
                    ),
            )
            .child(self.render_main_content(window, cx))
    }
}
