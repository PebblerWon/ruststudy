use crate::ui::app::AppState;
use eframe::egui::{Color32, RichText, Ui};
use rand::Rng;

pub struct WatchlistPanel {
    filter: String,
    symbols: Vec<WatchlistItem>,
}

pub struct WatchlistItem {
    pub symbol: String,
    pub price: f64,
    pub change_24h: f64,
}

impl WatchlistPanel {
    pub fn new() -> Self {
        let mut mock_items = Vec::new();
        let mut rng = rand::thread_rng();
        for i in 0..10 {
            mock_items.push(WatchlistItem {
                symbol: format!("BTC{}_usdt", i),
                price: rng.gen_range(80.0..100.0),
                change_24h: rng.gen_range(-10.0..10.0),
            });
        }
        WatchlistPanel {
            filter: String::from(""),
            symbols: mock_items,
        }
    }
    pub fn ui(&mut self, ui: &mut Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.filter);
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.strong("交易对");
            ui.add_space(ui.available_width() - 120.0);
            ui.strong("价格");
            ui.strong("24h%");
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for item in self.filtered_items() {
                ui.horizontal(|ui| {
                    let response = ui
                        .scope(|ui| {
                            ui.set_min_width(ui.available_width());

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&item.symbol).strong());
                                ui.add_space(ui.available_width() - 120.0);
                                ui.label(format!("{:.2}", item.price));

                                let change_color = if item.change_24h >= 0.0 {
                                    Color32::from_rgb(0, 200, 53)
                                } else {
                                    Color32::from_rgb(234, 57, 67)
                                };
                                ui.label(
                                    RichText::new(format!("{:+.2}%", item.change_24h))
                                        .color(change_color),
                                );
                            })
                        })
                        .response;
                });
            }
        });
    }

    fn filtered_items(&self) -> Vec<&WatchlistItem> {
        if self.filter.is_empty() {
            self.symbols.iter().collect()
        } else {
            self.symbols
                .iter()
                .filter(|item| {
                    item.symbol
                        .to_lowercase()
                        .contains(&self.filter.to_lowercase())
                })
                .collect()
        }
    }
}
