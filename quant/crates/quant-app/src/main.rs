mod ui;
use crate::ui::app::QuantApp;
use gpui_kit::{assets::Assets, *};

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(QuantApp::new))
            .expect("failed to open window");
    });
}
