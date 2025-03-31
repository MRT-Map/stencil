use bevy_egui::egui;

use crate::ui::panel::{
    dock::{DockWindow, TabViewer},
    toolbar::toolbar,
};

#[derive(Clone, Copy)]
pub struct Tilemap;

impl DockWindow for Tilemap {
    fn title(self) -> String {
        "Map".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        toolbar(ui, tab_viewer);
        *tab_viewer.pointer_within_tilemap = ui.rect_contains_pointer(ui.available_rect_before_wrap());
    }
    fn allowed_in_windows(self) -> bool {
        false
    }
    fn closeable(self) -> bool {
        false
    }
}
