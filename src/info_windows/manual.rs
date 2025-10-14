use serde::{Deserialize, Serialize};

use crate::{App, ui::popup::Popup};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct ManualPopup;

impl Popup for ManualPopup {
    fn id(&self) -> String {
        "manual".into()
    }

    fn title(&self) -> String {
        "Manual".into()
    }

    fn ui(&mut self, _app: &mut App, ui: &mut egui::Ui) -> bool {
        ui.label("Our online manual is available here:");
        ui.hyperlink("https://github.com/MRT-Map/stencil2/wiki");
        !ui.button("Close").clicked()
    }
}
