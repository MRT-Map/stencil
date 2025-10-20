use tracing::info;

use crate::{App, map::MapWindow, mode::EditorMode};

impl MapWindow {
    pub fn toolbar(&mut self, app: &mut App, ui: &mut egui::Ui) {
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

                if app.project.path.is_none() {
                    ui.label(
                        egui::RichText::new(" THIS IS A SCRATCHPAD - NOTHING WILL BE SAVED ")
                            .background_color(egui::Color32::LIGHT_RED)
                            .color(egui::Color32::BLACK),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui.button("Reset View").clicked() {
                        info!("Resetting view");
                        self.reset(app);
                    }
                    if let Some(prev_cursor_world_pos) = self.prev_cursor_world_pos {
                        ui.label(format!(
                            "x: {} z: {} \u{1f50d}: {:.2}",
                            prev_cursor_world_pos.x.round() as i32,
                            prev_cursor_world_pos.y.round() as i32,
                            self.zoom
                        ));
                    } else {
                        ui.label(format!("\u{1f50d}: {:.2}", self.zoom));
                    }

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
            }.into();
        }
    }
}
