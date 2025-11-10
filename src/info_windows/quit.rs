use serde::{Deserialize, Serialize};

use crate::{App, ui::popup::Popup};

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
            Some(|ctx: &egui::Context, _app: &mut App| {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }),
            Option::<fn(&_, &mut _)>::None,
        )
    }
}
