use tracing::info;

use crate::{App, component_actions::event::ComponentEv, map::MapWindow, project::pla3::PlaNode};

impl MapWindow {
    pub fn copy_selected_components(&mut self, app: &App) {
        self.clipboard = self
            .selected_components(&app.project.components)
            .into_iter()
            .cloned()
            .collect();
        if self.clipboard.is_empty() {
            info!("Nothing to copy");
        } else {
            info!(ids=?self.clipboard.iter().map(|a| &a.full_id).collect::<Vec<_>>(), "Copied components");
        }
    }
}

impl App {
    pub fn copy_selected_components(&mut self) {
        let map_window = self.ui.dock_layout.map_window_mut();
        map_window.clipboard = map_window
            .selected_components(&self.project.components)
            .into_iter()
            .cloned()
            .collect();
        if map_window.clipboard.is_empty() {
            info!("Nothing to copy");
        } else {
            info!(ids=?map_window.clipboard.iter().map(|a| &a.full_id).collect::<Vec<_>>(), "Copied components");
        }
    }
    pub fn cut_selected_components(&mut self, ctx: &egui::Context) {
        self.copy_selected_components();
        self.delete_selected_components(ctx);
    }
    pub fn paste_clipboard_components(&mut self, ctx: &egui::Context) {
        let map_window = self.ui.dock_layout.map_window();
        let Some(centre) =
            PlaNode::centre(map_window.clipboard.iter().flat_map(|a| a.nodes.clone()))
        else {
            info!("Nothing to paste");
            return;
        };
        let delta = map_window.cursor_world_pos.map_or_else(
            || geo::coord! { x: map_window.centre_coord.x.round() as i32, y: map_window.centre_coord.y.round() as i32 },
            |a| geo::coord! { x: a.x.round() as i32, y: a.y.round() as i32 }
        ) - centre;
        let components_to_add = map_window
            .clipboard
            .iter()
            .cloned()
            .map(|mut component| {
                component
                    .full_id
                    .namespace
                    .clone_from(&self.project.new_component_ns);
                component.full_id.id = self
                    .project
                    .components
                    .get_new_id(&self.project.new_component_ns);
                for node in &mut component.nodes {
                    *node += delta;
                }
                component
            })
            .collect::<Vec<_>>();
        let ids = components_to_add
            .iter()
            .map(|a| a.full_id.clone())
            .collect::<Vec<_>>();
        info!(?ids, "Pasted and selected components");
        self.run_event(ComponentEv::Create(components_to_add), ctx);
        self.ui.dock_layout.map_window_mut().selected_components = ids;
    }
}
