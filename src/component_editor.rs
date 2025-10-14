use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::{App, ui::dock::DockWindow};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ComponentEditorWindow;

impl DockWindow for ComponentEditorWindow {
    fn title(self) -> String {
        "Component Editor".into()
    }
    fn ui(self, app: &mut App, ui: &mut Ui) {
        ui.label("comp edit");
    }
}
