use gpui_kit::*;
mod ui;

use ui::app::StockApp;
fn main() {
    application().with_assets(assets::Assets).run(|cx| {
        init(cx);

        open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: 50.0.into(),
                        y: 50.0.into(),
                    },
                    size: Size {
                        width: 1200.0.into(),
                        height: 800.0.into(),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::new_static("Rust 量化交易")),
                    appears_transparent: Default::default(),
                    traffic_light_position: Default::default(),
                }),
                window_min_size: Some(Size {
                    width: px(1200.0),
                    height: px(800.0),
                }),
                ..WindowOptions::default()
            },
            cx,
            |_, cx| cx.new(|_| StockApp::new()),
        )
        .expect("Failed to open window");
    });
}
