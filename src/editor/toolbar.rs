use crate::{ComponentType, EditorState, HoveringOverGui};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use iyes_loopless::prelude::*;

pub fn ui(
    mut ctx: ResMut<EguiContext>,
    mut commands: Commands,
    mut hovering: ResMut<HoveringOverGui>,
    mut cv: Local<&'static str>,
) {
    let mut current_value = *cv;
    let panel = egui::TopBottomPanel::top("toolbar").show(ctx.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            if ui
                .selectable_value(&mut current_value, "", "Select")
                .clicked()
            {
                commands.insert_resource(NextState(EditorState::Idle))
            }
            if ui
                .selectable_value(&mut current_value, "point", "Point")
                .clicked()
            {
                commands.insert_resource(NextState(EditorState::CreatingComponent(
                    ComponentType::Point,
                )))
            }
            if ui
                .selectable_value(&mut current_value, "line", "Line")
                .clicked()
            {
                commands.insert_resource(NextState(EditorState::CreatingComponent(
                    ComponentType::Line,
                )))
            }
            if ui
                .selectable_value(&mut current_value, "area", "Area")
                .clicked()
            {
                commands.insert_resource(NextState(EditorState::CreatingComponent(
                    ComponentType::Area,
                )))
            }
            ui.selectable_value(&mut current_value, "edit_nodes", "Edit Nodes");
            ui.selectable_value(&mut current_value, "move", "Move Components");
            ui.selectable_value(&mut current_value, "rotate", "Rotate Components");
            ui.selectable_value(&mut current_value, "delete", "Delete Components");
        });
    });
    if panel.response.hovered() {
        hovering.0 = true
    }
    *cv = current_value;
}
