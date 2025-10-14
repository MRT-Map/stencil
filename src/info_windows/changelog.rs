use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use serde::{Deserialize, Serialize};

use crate::{App, ui::popup::Popup};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct ChangelogPopup;

impl Popup for ChangelogPopup {
    fn id(&self) -> String {
        "changelog".into()
    }

    fn title(&self) -> String {
        "Changelog".into()
    }

    fn window(&self) -> egui::Window<'static> {
        self.default_window().resizable(true)
    }

    fn ui(&mut self, _app: &mut App, ui: &mut egui::Ui) -> bool {
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() * 0.75)
            .show(ui, |ui| {
                let mut cache = CommonMarkCache::default();
                CommonMarkViewer::new().show(ui, &mut cache, include_str!("../../changelog.md"));
            });
        ui.separator();
        !ui.button("Close").clicked()
    }
}
