use std::borrow::Cow;
use serde::{Deserialize, Serialize};

use crate::{App, ui::popup::Popup};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct ManualPopup;

impl Popup for ManualPopup {
    fn id(&self) -> Cow<'static, str> {
        "manual".into()
    }

    fn title(&self) -> egui::WidgetText {
        "Manual".into()
    }

    fn ui(&mut self, _app: &mut App, ui: &mut egui::Ui) -> bool {
        ui.label("Our online manual is available here:");
        ui.hyperlink("https://github.com/MRT-Map/stencil3/wiki");
        !ui.button("Close").clicked()
    }
}
