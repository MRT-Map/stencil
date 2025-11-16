use crate::{App, component_actions::ComponentEv, map::MapWindow, mode::EditorMode};

impl MapWindow {
    fn move_components_by(&self, app: &mut App, delta: geo::Coord<i32>) {
        if delta == geo::Coord::zero() {
            return;
        }
        for component in self.selected_components_mut(&mut app.project.components) {
            for node in &mut component.nodes {
                *node += delta;
            }
        }
    }
    pub fn move_components(&mut self, app: &mut App, response: &egui::Response) {
        if app.mode != EditorMode::Nodes {
            if let Some(move_delta) = self.move_delta.take() {
                self.move_components_by(app, -move_delta);
            }
            return;
        }
        if response.drag_stopped_by(egui::PointerButton::Primary)
            && let Some(move_delta) = self.move_delta.take()
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
            app.add_event(ComponentEv::ChangeField {
                before,
                after,
                label: "nodes",
            });
            return;
        }
        if !response.dragged_by(egui::PointerButton::Primary)
            || (response.drag_started_by(egui::PointerButton::Primary)
                && self
                    .hovered_component
                    .as_ref()
                    .is_none_or(|a| self.selected_components.contains(a)))
        {
            return;
        }

        let new_move_delta = response.total_drag_delta().unwrap_or_default()
            * app.world_screen_ratio_with_current_basemap_at_zoom(self.zoom);
        let new_move_delta =
            geo::coord! { x: new_move_delta.x.round() as i32, y: new_move_delta.y.round() as i32 };

        let this_frame_delta = new_move_delta - self.move_delta.unwrap_or_default();
        self.move_delta = Some(new_move_delta);
        self.move_components_by(app, this_frame_delta);
    }
}
