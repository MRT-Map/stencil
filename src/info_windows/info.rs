use serde::{Deserialize, Serialize};

use crate::{App, ui::popup::Popup};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct InfoPopup;

impl Popup for InfoPopup {
    fn id(&self) -> String {
        "info".into()
    }

    fn title(&self) -> String {
        "Info".into()
    }

    fn ui(&mut self, _app: &mut App, ui: &mut egui::Ui) -> bool {
        ui.add(
            egui::Image::new(egui::include_image!("../../assets/ste-light.png"))
                .fit_to_exact_size(egui::vec2(975.0 / 4.0, 569.0 / 4.0)),
        );
        ui.label("Made by __7d for the MRT Mapping Services");
        ui.hyperlink_to("GitHub", "https://github.com/MRT-Map/stencil2");
        !ui.button("Close").clicked()
    }
}
