use serde::{Deserialize, Serialize};

use crate::{App, ui::dock::DockWindow};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ComponentEditorWindow;

impl DockWindow for ComponentEditorWindow {
    fn title(&self) -> String {
        "Component Editor".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        ui.label("comp edit");
    }
}
