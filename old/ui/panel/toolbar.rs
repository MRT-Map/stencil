use bevy_egui::egui;

use crate::{
    state::{ChangeStateEv, EditorState},
    ui::{map::mouse_nav::ScrollMode, panel::dock::PanelParams},
};

pub fn toolbar(ui: &mut egui::Ui, params: &mut PanelParams) -> egui::InnerResponse<()> {
    let PanelParams {
        editor_state,
        commands,
        mouse_pos_world,
        pending_tiles,
        zoom,
        misc_settings,
        ..
    } = params;
    let mut new_state = ***editor_state;
    let resp = egui::TopBottomPanel::top("toolbar").show_inside(ui, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            macro_rules! button {
                ($text:literal, $next_state:expr) => {
                    ui.selectable_value(&mut new_state, $next_state, $text)
                };
            }

            button!("Select", EditorState::Idle);

            ui.separator();
            button!("Edit Nodes", EditorState::EditingNodes);
            button!("Delete", EditorState::DeletingComponent);

            ui.separator();
            ui.label("Create...");
            button!("Point", EditorState::CreatingPoint);
            button!("Line", EditorState::CreatingLine);
            button!("Area", EditorState::CreatingArea);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                ui.label(format!(
                    "x: {} z: {} \u{1f50d}: {:.2}",
                    mouse_pos_world.round().x as i32,
                    -mouse_pos_world.round().y as i32,
                    zoom.0
                ));
                ui.separator();
                ui.label(format!("# Pending Tiles: {}", pending_tiles.0.len()));
                ui.separator();
            });
        });
    });
    if new_state != ***editor_state {
        commands.trigger(ChangeStateEv(new_state));
        params.status.set(match new_state {
            EditorState::Idle => match misc_settings.scroll_mode {
                ScrollMode::Zoom => "Idle: L-Click to select component. L-Click-drag to pan. Scroll to zoom.",
                ScrollMode::Pan => "Idle: L-Click to select component. Scroll or L-Click-drag to pan. Shift and scroll to pan horizontally. Ctrl and scroll to zoom.",
            },
            EditorState::EditingNodes => "Editing nodes: R-click and drag circles to create node. R-click large circle without dragging to delete node.",
            EditorState::CreatingPoint => "Creating points: L-click to create point.",
            EditorState::CreatingLine => "Creating lines: L-click to start and continue line, L-click previous node to undo it. R-click to end. Alt to snap to angle.",
            EditorState::CreatingArea => "Creating areas: L-click to start and continue line, L-click previous node to undo it. L-click first node or R-click to end. Alt to snap to angle.",
            EditorState::DeletingComponent => "Deleting components: L-click to delete node.",
            _ => ""
        });
    }
    resp
}
