use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::{App, ui::dock::DockWindow};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct MapWindow;

impl DockWindow for MapWindow {
    fn title(self) -> String {
        "Map".into()
    }
    fn allowed_in_windows(self) -> bool {
        false
    }
    fn is_closeable(self) -> bool {
        false
    }
    fn ui(self, app: &mut App, ui: &mut Ui) {
        ui.label("tilemap");
    }
}
