use tracing::info;

use crate::{App, component_actions::ComponentEv, map::MapWindow, mode::EditorMode};

impl MapWindow {
    fn move_selected_components_by(&self, delta: geo::Coord<i32>, app: &mut App) {
        if delta == geo::Coord::zero() {
            return;
        }
        for component in self.selected_components_mut(&mut app.project.components) {
            for node in &mut component.nodes {
                *node += delta;
            }
        }
    }
    pub fn move_components(&self, app: &mut App, response: &egui::Response) {
        let id = "move delta".into();
        let mut move_delta = response
            .ctx
            .memory_mut(|m| m.data.get_temp::<geo::Coord<i32>>(id));
        let set_move_delta = |move_delta: Option<geo::Coord<i32>>| {
            response.ctx.memory_mut(|m| {
                if let Some(move_delta) = move_delta {
                    m.data.insert_temp(id, move_delta);
                } else {
                    m.data.remove::<geo::Coord<i32>>(id);
                }
            });
        };

        if app.mode != EditorMode::Nodes {
            if let Some(move_delta) = move_delta.take() {
                info!(?move_delta, "Move cancelled");
                self.move_selected_components_by(-move_delta, app);
            }
            set_move_delta(move_delta);
            return;
        }
        if response.drag_stopped_by(egui::PointerButton::Primary)
            && let Some(move_delta) = move_delta.take()
        {
            let after = self
                .selected_components(&app.project.components)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let before = after
                .iter()
                .map(|component| {
                    let mut component = component.to_owned();
                    for node in &mut component.nodes {
                        *node += -move_delta;
                    }
                    component
                })
                .collect();

            info!(?move_delta, "Move finished");
            app.add_event(ComponentEv::ChangeField {
                before,
                after,
                label: "move".into(),
            });
            set_move_delta(None);
            return;
        }
        if !response.dragged_by(egui::PointerButton::Primary)
            || (response.drag_started_by(egui::PointerButton::Primary)
                && self
                    .hovered_component
                    .as_ref()
                    .is_none_or(|a| !self.selected_components.contains(a)))
        {
            set_move_delta(move_delta);
            return;
        }

        if response.drag_started_by(egui::PointerButton::Primary) {
            info!("Move started");
        }

        let new_move_delta = response.total_drag_delta().unwrap_or_default()
            * app.world_screen_ratio_with_current_basemap_at_zoom(self.zoom);
        let new_move_delta =
            geo::coord! { x: new_move_delta.x.round() as i32, y: new_move_delta.y.round() as i32 };

        let this_frame_delta = new_move_delta - move_delta.unwrap_or_default();
        set_move_delta(Some(new_move_delta));
        self.move_selected_components_by(this_frame_delta, app);
    }
}
