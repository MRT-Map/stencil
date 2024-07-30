use bevy_egui::egui;

use crate::{
    state::{ChangeStateEv, EditorState},
    ui::panel::dock::{PanelParams, TabViewer},
};

pub fn toolbar(ui: &mut egui::Ui, tab_viewer: &mut TabViewer) -> egui::InnerResponse<()> {
    let PanelParams {
        editor_state,
        commands,
        ..
    } = tab_viewer.params;
    let mut new_state = ***editor_state;
    let resp = egui::TopBottomPanel::top("toolbar").show_inside(ui, |ui| {
        egui::menu::bar(ui, |ui| {
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
        });
    });
    if new_state != ***editor_state {
        //commands.trigger(ChangeStateEv(new_state)));
        commands.trigger(ChangeStateEv(new_state));
        tab_viewer.params.status.0 = match new_state {
            EditorState::Idle => "Idle: L-Click to select component, or drag to pan. Zoom to scroll.",
            EditorState::EditingNodes => "Editing nodes: R-click and drag circles to create node. R-click large circle without dragging to delete node.",
            EditorState::CreatingPoint => "Creating points: L-click to create point.",
            EditorState::CreatingLine => "Creating lines: L-click to start and continue line, L-click previous node to undo it. R-click to end. Alt to snap to angle.",
            EditorState::CreatingArea => "Creating areas: L-click to start and continue line, L-click previous node to undo it. L-click first node or R-click to end. Alt to snap to angle.",
            EditorState::DeletingComponent => "Deleting components: L-click to delete node.",
            _ => ""
        }.into();
    }
    resp
}
