//! 最小复现：group_hover 放在 group 元素自身 vs 放在子元素上，行为是否一致。
//! 测量方式：group_hover 将元素尺寸从 10 变为 20，用 canvas paint 回调记录实际生效的宽度。

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnyWindowHandle, AppContext as _, Context, IntoElement, InteractiveElement, ParentElement,
    Pixels, Render, Styled, TestAppContext, Window, canvas, div, point, px,
};

/// 场景 A：group("card") 与 group_hover("card") 挂在同一个元素上（overview_panel.rs card() 的写法）
struct SelfGroupHover {
    painted_width: Rc<Cell<Pixels>>,
}

impl Render for SelfGroupHover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let painted_width = self.painted_width.clone();
        div().size_full().child(
            div()
                .ml(px(20.))
                .mt(px(20.))
                .size(px(10.))
                .relative()
                .group("card")
                .group_hover("card", |style| style.size(px(20.)))
                .child(
                    div().absolute().size_full().child(canvas(
                        move |bounds, _, _| painted_width.set(bounds.size.width),
                        |_, _, _, _| {},
                    )),
                ),
        )
    }
}

/// 场景 B：group 在父元素，group_hover 在子元素
/// （对应 render_snapshot_card 里 name div 的写法，也是 gpui 官方测试的用法）
struct ChildGroupHover {
    painted_width: Rc<Cell<Pixels>>,
}

impl Render for ChildGroupHover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let painted_width = self.painted_width.clone();
        div().size_full().child(
            div()
                .ml(px(20.))
                .mt(px(20.))
                .size(px(10.))
                .relative()
                .group("card")
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size(px(10.))
                        .group_hover("card", |style| style.size(px(20.)))
                        .child(canvas(
                            move |bounds, _, _| painted_width.set(bounds.size.width),
                            |_, _, _, _| {},
                        )),
                ),
        )
    }
}

#[gpui::test]
fn scenario_a_group_hover_on_group_element_itself(cx: &mut TestAppContext) {
    let painted_width = Rc::new(Cell::new(px(0.)));
    let window = cx.add_window({
        let painted_width = painted_width.clone();
        move |_, _| SelfGroupHover { painted_width }
    });
    let window = AnyWindowHandle::from(window);

    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    println!("场景A 基线（未 hover）宽度 = {:?}", painted_width.get());
    assert_eq!(painted_width.get(), px(10.0), "baseline should be 10");

    // 鼠标移入 (20,20)-(30,30) 的卡片
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(25.), px(25.)), cx);
    })
    .unwrap();
    println!("场景A hover 后宽度 = {:?}", painted_width.get());

    // 移出，观察恢复
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(5.), px(5.)), cx);
    })
    .unwrap();
    println!("场景A 移出后宽度 = {:?}", painted_width.get());

    assert_eq!(
        painted_width.get(),
        px(10.0),
        "场景A: group_hover 放在 group 元素自身上，hover 后样式应恢复为 10"
    );
}

/// 场景 C：官方测试原样——子元素带 .id()（stateful），group_hover 在子元素上
struct OfficialLike {
    stateful_width: Rc<Cell<Pixels>>,
}

impl Render for OfficialLike {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let stateful_width = self.stateful_width.clone();
        div().size_full().child(
            div()
                .ml(px(20.))
                .mt(px(20.))
                .size(px(50.))
                .relative()
                .group("hover-group")
                .child(
                    div()
                        .id("stateful-group-hover-target")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size(px(10.))
                        .group_hover("hover-group", |style| style.size(px(20.)))
                        .child(canvas(
                            move |bounds, _, _| stateful_width.set(bounds.size.width),
                            |_, _, _, _| {},
                        )),
                ),
        )
    }
}

/// 场景 D：与场景 A 同构，但卡片加 .id()（stateful），group_hover 放自身上
struct SelfGroupHoverWithId {
    painted_width: Rc<Cell<Pixels>>,
}

impl Render for SelfGroupHoverWithId {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let painted_width = self.painted_width.clone();
        div().size_full().child(
            div()
                .id("self-card")
                .ml(px(20.))
                .mt(px(20.))
                .size(px(10.))
                .relative()
                .group("card")
                .group_hover("card", |style| style.size(px(20.)))
                .child(
                    div().absolute().size_full().child(canvas(
                        move |bounds, _, _| painted_width.set(bounds.size.width),
                        |_, _, _, _| {},
                    )),
                ),
        )
    }
}

#[gpui::test]
fn scenario_b_group_hover_on_child_inside_group(cx: &mut TestAppContext) {
    let painted_width = Rc::new(Cell::new(px(0.)));
    let window = cx.add_window({
        let painted_width = painted_width.clone();
        move |_, _| ChildGroupHover { painted_width }
    });
    let window = AnyWindowHandle::from(window);

    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    println!("场景B 基线（未 hover）宽度 = {:?}", painted_width.get());
    assert_eq!(painted_width.get(), px(10.0), "baseline should be 10");

    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(25.), px(25.)), cx);
    })
    .unwrap();
    println!("场景B hover 后宽度 = {:?}", painted_width.get());
    assert_eq!(
        painted_width.get(),
        px(20.0),
        "场景B: 子元素上的 group_hover 应该生效（20）"
    );
}

#[gpui::test]
fn scenario_c_official_stateful_child(cx: &mut TestAppContext) {
    let stateful_width = Rc::new(Cell::new(px(0.)));
    let window = cx.add_window({
        let stateful_width = stateful_width.clone();
        move |_, _| OfficialLike { stateful_width }
    });
    let window = AnyWindowHandle::from(window);

    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    println!("场景C 基线（未 hover）宽度 = {:?}", stateful_width.get());
    assert_eq!(stateful_width.get(), px(10.0), "baseline should be 10");

    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(25.), px(25.)), cx);
    })
    .unwrap();
    println!("场景C hover 后宽度 = {:?}", stateful_width.get());
    assert_eq!(
        stateful_width.get(),
        px(20.0),
        "场景C: 带 .id() 的子元素 group_hover 应该生效（20）"
    );
}

/// 场景 E：无状态 div（无 .id()）用 .hover() —— 对应 overview_panel.rs card() 当前写法
/// 观测 paint_count（canvas paint 回调执行次数 = 实际重绘次数）：
/// 核心问题：hover 进入/离开是否【自动触发】重绘，还是只能等下一次外力重绘（轮询等）。
struct StatelessHover {
    paint_count: Rc<Cell<u32>>,
}

impl Render for StatelessHover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let paint_count = self.paint_count.clone();
        div().size_full().child(
            div()
                .ml(px(20.))
                .mt(px(20.))
                .size(px(10.))
                .relative()
                .hover(|style| style.shadow_md())
                .child(
                    div().absolute().size_full().child(canvas(
                        move |_, _, _| paint_count.set(paint_count.get() + 1),
                        |_, _, _, _| {},
                    )),
                ),
        )
    }
}

/// 场景 F：带 .id() 的 div 用 .hover() —— 对照组
struct StatefulHover {
    paint_count: Rc<Cell<u32>>,
}

impl Render for StatefulHover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let paint_count = self.paint_count.clone();
        div().size_full().child(
            div()
                .id("hover-card")
                .ml(px(20.))
                .mt(px(20.))
                .size(px(10.))
                .relative()
                .hover(|style| style.shadow_md())
                .child(
                    div().absolute().size_full().child(canvas(
                        move |_, _, _| paint_count.set(paint_count.get() + 1),
                        |_, _, _, _| {},
                    )),
                ),
        )
    }
}

#[gpui::test]
fn scenario_e_stateless_hover_no_auto_redraw(cx: &mut TestAppContext) {
    let paint_count = Rc::new(Cell::new(0));
    let window = cx.add_window({
        let paint_count = paint_count.clone();
        move |_, _| StatelessHover { paint_count }
    });
    let window = AnyWindowHandle::from(window);

    // 初始绘制（可能触发多次 paint，用差分观测）
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let c0 = paint_count.get();
    println!("场景E 基线 paint_count = {}", c0);

    // 鼠标移入卡片，不手动绘制 —— hover 事件是否自动触发重绘？
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(25.), px(25.)), cx);
    })
    .unwrap();
    let c1 = paint_count.get();
    println!("场景E 移入后 paint_count = {}（基线 {}）", c1, c0);
    assert_eq!(
        c1, c0,
        "场景E: 无状态 div 的 .hover() 移入后没有触发任何重绘 —— 卡顿根源"
    );

    // 外力重绘（模拟轮询数据更新触发的下一帧）：此时 paint 期样式判定才生效
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let c2 = paint_count.get();
    println!("场景E 外力重绘后 paint_count = {}", c2);
    assert!(c2 > c1, "外力重绘后计数应增加");

    // 移出：同样不触发重绘，高亮残留
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(5.), px(5.)), cx);
    })
    .unwrap();
    let c3 = paint_count.get();
    println!("场景E 移出后 paint_count = {}", c3);
    assert_eq!(
        c3, c2,
        "场景E: 移出后同样没有自动重绘 —— 高亮残留直到下一次外力重绘"
    );

    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert!(paint_count.get() > c3);
}

#[gpui::test]
fn scenario_f_stateful_hover_auto_redraw(cx: &mut TestAppContext) {
    let paint_count = Rc::new(Cell::new(0));
    let window = cx.add_window({
        let paint_count = paint_count.clone();
        move |_, _| StatefulHover { paint_count }
    });
    let window = AnyWindowHandle::from(window);

    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let c0 = paint_count.get();
    println!("场景F 基线 paint_count = {}", c0);

    // 移入：listener 在状态变化瞬间 cx.notify → 自动重绘
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(25.), px(25.)), cx);
    })
    .unwrap();
    let c1 = paint_count.get();
    println!("场景F 移入后 paint_count = {}（基线 {}）", c1, c0);
    assert!(
        c1 > c0,
        "场景F: 带 .id() 的 .hover() 移入后应自动触发重绘（事件驱动，即时）"
    );

    // 移出：同样自动重绘
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(5.), px(5.)), cx);
    })
    .unwrap();
    let c2 = paint_count.get();
    println!("场景F 移出后 paint_count = {}", c2);
    assert!(c2 > c1, "移出后应自动重绘恢复");
}

#[gpui::test]
fn scenario_d_self_group_hover_with_element_id(cx: &mut TestAppContext) {
    let painted_width = Rc::new(Cell::new(px(0.)));
    let window = cx.add_window({
        let painted_width = painted_width.clone();
        move |_, _| SelfGroupHoverWithId { painted_width }
    });
    let window = AnyWindowHandle::from(window);

    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    println!("场景D 基线（未 hover）宽度 = {:?}", painted_width.get());
    assert_eq!(painted_width.get(), px(10.0), "baseline should be 10");

    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(25.), px(25.)), cx);
    })
    .unwrap();
    println!("场景D hover 后宽度 = {:?}", painted_width.get());
    assert_eq!(
        painted_width.get(),
        px(20.0),
        "场景D: 带 .id() 的卡片上 group_hover 放自身应该生效（20）"
    );

    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(5.), px(5.)), cx);
    })
    .unwrap();
    println!("场景D 移出后宽度 = {:?}", painted_width.get());
    assert_eq!(painted_width.get(), px(10.0), "移出后应恢复（10）");
}
