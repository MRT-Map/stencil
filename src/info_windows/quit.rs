use serde::{Deserialize, Serialize};

use crate::{App, event::Events, info_windows::InfoWindowEv, ui::popup::Popup};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct QuitPopup;

impl Popup for QuitPopup {
    fn id(&self) -> String {
        "confirm_quit".into()
    }

    fn title(&self) -> String {
        "Are you sure you want to exit?".into()
    }

    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) -> bool {
        self.confirm_ui(
            app,
            ui,
            "You may have unsaved changes",
            Some(InfoWindowEv::Quit { confirm: true }),
            Option::<Events>::None,
        )
    }
}
