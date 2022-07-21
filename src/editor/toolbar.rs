use crate::{ComponentType, EditorState, HoveringOverGui};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use iyes_loopless::prelude::*;

pub fn ui(mut ctx: ResMut<EguiContext>, mut commands: Commands, mut hovering: ResMut<HoveringOverGui>) {
    let mut current_value = "";
    let panel = egui::TopBottomPanel::top("toolbar").show(ctx.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            if ui.selectable_value(&mut current_value, "point", "Point").clicked() {
                commands.insert_resource(NextState(EditorState::CreatingComponent(ComponentType::Point)))
            } // TODO more down there
            ui.selectable_value(&mut current_value, "line", "Line");
            ui.selectable_value(&mut current_value, "area", "Area");
            ui.selectable_value(&mut current_value, "edit_nodes", "Edit Nodes");
            ui.selectable_value(&mut current_value, "move", "Move Components");
            ui.selectable_value(&mut current_value, "rotate", "Rotate Components");
            ui.selectable_value(&mut current_value, "delete", "Delete Components");
        });
    });
    if panel.response.hovered() {
        hovering.0 = true
    }
}
