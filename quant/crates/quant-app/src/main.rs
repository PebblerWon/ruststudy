mod ui;
use crate::ui::app::QuantApp;
use eframe::{NativeOptions, egui};
use egui_chinese_font::setup_chinese_fonts;

#[tokio::main]
async fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter("quant_app=info")
        .init();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RustQuant - 桌面量化交易系统")
            .with_inner_size([1280.0, 800.0])
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RustQuant",
        options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx)?;
            Ok(Box::new(QuantApp::new(1.23)))
        }),
    )
}
