use bevy::prelude::Resource;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};

use crate::ui::panel::{
    dock::{DockWindow, PanelParams},
    toolbar::toolbar,
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Tilemap;

#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct PointerWithinTilemap;

impl DockWindow for Tilemap {
    fn title(self) -> String {
        "Map".into()
    }
    fn ui(self, params: &mut PanelParams, ui: &mut egui::Ui) {
        toolbar(ui, params);
        let PanelParams { commands, .. } = params;
        if ui.rect_contains_pointer(ui.available_rect_before_wrap()) {
            commands.init_resource::<PointerWithinTilemap>();
        } else {
            commands.remove_resource::<PointerWithinTilemap>();
        }
    }
    fn allowed_in_windows(self) -> bool {
        false
    }
    fn closeable(self) -> bool {
        false
    }
}
