use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use iyes_loopless::prelude::*;

use crate::{
    misc::{Action, ChangeStateAct, EditorState},
    pla2::component::ComponentType,
    ui::{component_panel::PrevNamespaceUsed, HoveringOverGui},
};

#[allow(clippy::needless_pass_by_value)]
pub fn ui_sy(
    state: Res<CurrentState<EditorState>>,
    mut ctx: ResMut<EguiContext>,
    mut actions: EventWriter<Action>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
) {
    let mut new_state = state.0;
    let panel = egui::TopBottomPanel::top("toolbar").show(ctx.ctx_mut(), |ui| {
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
            button!(
                "Point",
                EditorState::CreatingComponent(ComponentType::Point)
            );
            button!("Line", EditorState::CreatingComponent(ComponentType::Line));
            button!("Area", EditorState::CreatingComponent(ComponentType::Area));
        });
    });
    if panel.response.hovered() {
        hovering_over_gui.0 = true;
    }
    if new_state != state.0 {
        actions.send(Box::new(ChangeStateAct(new_state)));
    }
}
