use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{App, ui::popup::Popup};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct MemorialPopup;

impl Popup for MemorialPopup {
    fn id(&self) -> Cow<'static, str> {
        "memorial".into()
    }

    fn title(&self) -> egui::WidgetText {
        "Stencil v1/v2 Memorial".into()
    }

    fn ui(&mut self, _app: &mut App, ui: &mut egui::Ui) -> bool {
        ui.heading("Stencil v1 2021-09-08 — 2022-12-25");
        ui.heading("Stencil v2 2022-12-25 — 202?-??-??");
        ui.small("Did you know? Until v3, Stencil never had a copy/paste feature");
        !ui.button("Close").clicked()
    }
}
