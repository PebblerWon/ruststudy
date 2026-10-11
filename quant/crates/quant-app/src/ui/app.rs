use eframe::{App, egui};
use egui::Color32;
use quant_data::common::models::Interval;

use crate::ui::control_panel::WatchlistPanel;

pub struct QuantApp {
    pub value: f64,
    state: AppState,
    watchlist: WatchlistPanel,
}
impl QuantApp {
    pub fn new(v: f64) -> Self {
        QuantApp {
            value: v,
            state: AppState {
                is_connected: true,
                selected_symbol: "".to_string(),
                current_interval: Interval::M1,
                last_update_time: Some(chrono::Utc::now().timestamp()),
                available_symbols: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            },
            watchlist: WatchlistPanel::new(),
        }
    }
    fn toolbar_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("交易对：");

            egui::ComboBox::from_id_salt("symbol_select")
                .selected_text(&self.state.selected_symbol)
                .show_ui(ui, |ui| {
                    for symbol in &self.state.available_symbols {
                        ui.selectable_value(
                            &mut self.state.selected_symbol,
                            symbol.clone(),
                            symbol,
                        );
                    }
                });

            ui.separator();

            ui.label("周期：");
            for interval in [
                Interval::M1,
                Interval::M5,
                Interval::M15,
                Interval::H1,
                Interval::H4,
                Interval::D1,
            ] {
                let label = interval.as_str();
                let is_selected = self.state.current_interval == interval;
                if ui.selectable_label(is_selected, label).clicked() {
                    self.state.current_interval = interval;
                    // self.state.load_kline_data();
                }
            }
        });
    }

    fn status_bar_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (dot_color, status_text) = if self.state.is_connected {
                (Color32::from_rgb(0, 200, 83), "已连接")
            } else {
                (Color32::from_rgb(234, 57, 67), "未连接")
            };
            ui.add(egui::Label::new(egui::RichText::new("●").color(dot_color)));

            ui.label(status_text);
            ui.separator();

            // 最新价格
            ui.label(format!("{}: {:.2}", "13:54:20", 35.98));
            ui.separator();
            if let Some(ts) = self.state.last_update_time {
                let time_str = chrono::DateTime::from_timestamp_millis(ts)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_default();
                ui.label(format!("更新：{}", time_str));
            }
            ui.separator();
            ui.label(format!("K 线：{} 条", 10));
        });
    }
}
impl eframe::App for QuantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.toolbar_ui(ui);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.status_bar_ui(ui);
        });

        egui::TopBottomPanel::bottom("indicator_panel")
            .default_height(200.0)
            .show(ctx, |ui| {
                // self.indicator_panel_ui(ui, &mut self.state);
            });

        egui::SidePanel::left("watchlist")
            .default_width(250.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                self.watchlist.ui(ui, &mut self.state);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            //
        });
    }
}

pub struct AppState {
    pub selected_symbol: String,
    pub current_interval: Interval,
    pub available_symbols: Vec<String>,
    pub is_connected: bool,
    pub last_update_time: Option<i64>,
}
