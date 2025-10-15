use serde::{Deserialize, Serialize};

use crate::{App, ui::dock::DockWindow};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct MapWindow;

impl DockWindow for MapWindow {
    fn title(&self) -> String {
        "Map".into()
    }
    fn allowed_in_windows(&self) -> bool {
        false
    }
    fn is_closeable(&self) -> bool {
        false
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        ui.label("tilemap");
    }
}
