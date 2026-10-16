mod ui;
use crate::ui::app::QuantApp;
use gpui_kit::{assets::Assets, *};

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                ..WindowOptions::default()
            },
            cx,
            |window, cx| cx.new(|cx| QuantApp::new(window, cx)),
        )
        .expect("failed to open window");
    });
}
