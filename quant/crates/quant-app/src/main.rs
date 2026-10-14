mod ui;
use crate::ui::app::QuantApp;
use gpui_kit::*;

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(QuantApp::new))
            .expect("failed to open window");
    });
}
