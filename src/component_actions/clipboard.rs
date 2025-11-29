use tracing::info;

use crate::{
    App, component_actions::event::ComponentEv, coord_conversion::CoordConversionExt,
    project::pla3::PlaNode,
};

impl App {
    pub fn copy_selected_components(&mut self, ctx: &egui::Context) {
        self.ui.map.clipboard = self
            .map_selected_components()
            .into_iter()
            .cloned()
            .collect();

        self.status_on_copy(ctx);
    }
    pub fn cut_selected_components(&mut self, ctx: &egui::Context) {
        self.copy_selected_components(ctx);
        self.delete_selected_components(ctx);

        self.status_on_cut(ctx);
    }
    pub fn paste_clipboard_components(&mut self, ctx: &egui::Context) {
        let Some(centre) =
            PlaNode::centre(self.ui.map.clipboard.iter().flat_map(|a| a.nodes.clone()))
        else {
            self.status_on_paste(&[], ctx);
            return;
        };
        let delta = self.ui.map.cursor_world_pos.map_or_else(
            || self.ui.map.centre_coord.to_geo_coord_i32(),
            CoordConversionExt::to_geo_coord_i32,
        ) - centre;
        let components_to_add = self
            .ui
            .map
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
        self.status_on_paste(&components_to_add, ctx);
        self.run_event(ComponentEv::Create(components_to_add), ctx);
        self.ui.map.selected_components = ids;
    }
}
