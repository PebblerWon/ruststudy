use std::sync::Arc;

use anyhow::{Error, Ok};
use gpui_kit::*;
mod bridge;
mod ui;
mod utils;

use ui::app::StockApp;

use crate::bridge::DataBridge;
fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let bridge = Arc::new(DataBridge::new()?);
    application().with_assets(assets::Assets).run(move |cx| {
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
            |window, cx| cx.new(|cx| StockApp::new(window, cx, bridge.clone())),
        )
        .expect("Failed to open window");
    });
    Ok(())
}
