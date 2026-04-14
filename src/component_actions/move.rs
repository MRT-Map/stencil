use tracing::info;

use crate::{
    App, component_actions::event::ComponentEv, map::MapWindow, mode::EditorMode,
    pointer::ResponsePointerExt,
};

impl MapWindow {
    // todo glitch when mouse leaves window
    pub fn move_components(app: &mut App, response: &egui::Response) {
        if app.mode != EditorMode::Select {
            if let Some(origin_world_pos) = app.ui.map.comp_move_origin_world_pos.take() {
                info!(?origin_world_pos, "Move cancelled");
            }
            return;
        }
        if response.drag_stopped_by2(egui::PointerButton::Primary)
            && !app.ui.map.selected.is_empty()
            && let Some(move_delta) = app.ui.map.comp_move_delta()
        {
            let before = app
                .map_selected_components()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let after = before
                .iter()
                .map(|component| {
                    let mut component = component.to_owned();
                    component.nodes += move_delta;
                    component
                })
                .collect();

            info!(?move_delta, "Move finished");
            app.status_on_move_finish(move_delta, &response.ctx);
            app.run_event(
                ComponentEv::ChangeField {
                    before,
                    after,
                    label: "move".into(),
                },
                &response.ctx,
            );
            app.ui.map.comp_move_origin_world_pos = None;
            return;
        }
        if !response.dragged_by2(egui::PointerButton::Primary)
            || (response.drag_started_by2(egui::PointerButton::Primary)
                && app
                    .ui
                    .map
                    .hovered_component
                    .as_ref()
                    .is_none_or(|a| !app.ui.map.is_selected(a)))
        {
            app.ui.map.comp_move_origin_world_pos = None;
            return;
        }

        if response.drag_started_by2(egui::PointerButton::Primary)
            && response.ctx.input(|i| i.modifiers.command)
        {
            info!("Move started");
            app.ui.map.comp_move_origin_world_pos = app.ui.map.cursor_world_pos;
        }

        if let Some(move_delta) = app.ui.map.comp_move_delta() {
            app.status_on_move(move_delta, &response.ctx);
        }
    }
}
