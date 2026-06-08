use std::collections::HashMap;

use crate::{
    App, component_actions::event::ComponentEv, notif, project::pla3::PlaNodeWorldVec,
    utils::coord::CoordFrom,
};

impl App {
    #[tracing::instrument(skip_all)]
    pub fn copy_selected_components(&mut self) {
        self.ui.map.clipboard = self
            .map_selected_components()
            .into_iter()
            .cloned()
            .collect();

        self.status_on_copy();
    }
    #[tracing::instrument(skip_all)]
    pub fn cut_selected_components(&mut self) {
        self.copy_selected_components();
        self.delete_selected_components();

        self.status_on_cut();
    }
    #[tracing::instrument(skip_all)]
    pub fn paste_clipboard_components(&mut self) {
        let Some(new_component_ns) = &self.project.new_component_ns else {
            notif!(info "Create or load a namespace first");
            return;
        };
        let Some(centre) = self
            .ui
            .map
            .clipboard
            .iter()
            .flat_map(|a| &a.nodes)
            .copied()
            .collect::<PlaNodeWorldVec>()
            .map(egui::Pos2::coord_from)
            .centre()
            .map(geo::Coord::<i32>::coord_from)
        else {
            self.status_on_paste(&[]);
            return;
        };
        let delta = geo::Coord::<i32>::coord_from(
            self.ui
                .map
                .cursor_world_pos
                .unwrap_or(self.ui.map.centre_coord),
        ) - centre;
        let components_to_add = self
            .ui
            .map
            .clipboard
            .iter()
            .cloned()
            .map(|mut component| {
                component.full_id.namespace.clone_from(new_component_ns);
                component.full_id.id = self.project.components.get_new_id(new_component_ns);
                component.nodes += delta;
                component
            })
            .collect::<Vec<_>>();

        let ids = components_to_add
            .iter()
            .map(|a| (a.full_id.clone(), Vec::new()))
            .collect::<HashMap<_, _>>();
        self.status_on_paste(&components_to_add);
        self.run_event(ComponentEv::Create(components_to_add));
        self.ui.map.selected = ids;
    }
    #[tracing::instrument(skip_all)]
    pub fn map_clear_clipboard(&mut self) {
        self.ui.map.clipboard.clear();
        self.status_on_clear_clipboard();
    }
}
