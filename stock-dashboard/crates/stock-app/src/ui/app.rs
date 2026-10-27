use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
    SidebarToggleButton,
};
use gpui_kit::component::{Icon, IconName};
use gpui_kit::prelude::FluentBuilder as _;

use gpui_kit::*;
pub struct StockAppState {}

impl StockAppState {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct StockApp {
    collapsed: bool,
    description: String,
    menus: Vec<StockAppMenuItem>,
}
pub struct StockAppMenuItem {
    label: String,
}
impl StockApp {
    pub fn new() -> Self {
        let menus = vec![
            StockAppMenuItem {
                label: "总览".into(),
            },
            StockAppMenuItem {
                label: "榜单".into(),
            },
        ];
        Self {
            collapsed: false,
            description: menus[0].label.clone(),
            menus,
        }
    }

    fn render_main_content(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let icon_collapsed = self.collapsed;

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
                    .child(self.description.clone()),
            )
    }

    fn render_sidebar(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SidebarGroup<SidebarMenu> {
        SidebarGroup::new("A股看板").child(SidebarMenu::new().children(self.menus.iter().map(
            |menu_item| {
                let label = menu_item.label.clone();
                SidebarMenuItem::new(&menu_item.label)
                    .icon(IconName::LayoutDashboard)
                    .active(self.description == label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.description = label.clone();
                        cx.notify();
                    }))
            },
        )))
        // SidebarGroup::new("A股看板").child(
        //     SidebarMenu::new().child(
        //         SidebarMenuItem::new("总览")
        //             .icon(IconName::LayoutDashboard)
        //             .on_click(cx.listener(|this, _, _, cx| {
        //                 this.description = "总览".into();
        //                 cx.notify();
        //             })),
        //     ),
        // )
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
