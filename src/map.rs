use serde::{Deserialize, Serialize};

use crate::{App, mode::EditorMode, ui::dock::DockWindow};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MapWindow {
    tiles: walkers::HttpTiles,
    map_memory: walkers::MapMemory,
}

impl Default for MapWindow {
    fn new(ctx: &egui::Context) -> Self {
        Self {
            tiles: walkers::HttpTiles::new(egui::OpenStreetMap, ctx),
            map_memory: walkers::MapMemory::default(),
        }
    }
}

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
        self.toolbar(app, ui)
    }
}

impl MapWindow {
    pub fn toolbar(app: &mut App, ui: &mut egui::Ui) {
        let old_mode = app.mode;
        egui::TopBottomPanel::top("toolbar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                macro_rules! button {
                    ($text:literal, $next_state:expr) => {
                        ui.selectable_value(&mut app.mode, $next_state, $text)
                    };
                }

                button!("Select", EditorMode::Select);
                button!("Nodes", EditorMode::Nodes);

                ui.separator();
                ui.label("Create...");
                button!("Point", EditorMode::CreatePoint);
                button!("Line", EditorMode::CreateLine);
                button!("Area", EditorMode::CreateArea);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.label(format!(
                        "x: {} z: {} \u{1f50d}: {:.2}",
                        0,
                        0,
                        0 // mouse_pos_world.round().x as i32,
                          // -mouse_pos_world.round().y as i32,
                          // zoom.0
                    ));
                    ui.separator();
                    ui.label(format!(
                        "# Pending Tiles: {}",
                        0 /*pending_tiles.0.len()*/
                    ));
                    ui.separator();
                });
            });
        });

        if old_mode != app.mode {
            app.ui.status = match app.mode {
                EditorMode::Select => "Select: L-Click to select component. Scroll to pan. Shift and scroll to pan horizontally. Ctrl and scroll to zoom.",
                EditorMode::Nodes => "Editing nodes: R-click and drag circles to create node. R-click large circle without dragging to delete node.",
                EditorMode::CreatePoint => "Creating points: L-click to create point.",
                EditorMode::CreateLine => "Creating lines: L-click to start and continue line, L-click previous node to undo it. R-click to end. Alt to snap to angle.",
                EditorMode::CreateArea => "Creating areas: L-click to start and continue line, L-click previous node to undo it. L-click first node or R-click to end. Alt to snap to angle.",
                _ => ""
            }.into();
        }
    }
}
