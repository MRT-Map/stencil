use crate::{App, component_actions::event::ComponentEv};

impl App {
    #[tracing::instrument(skip_all)]
    pub fn delete_selected_components(&mut self, ctx: &egui::Context) {
        let components = self
            .map_selected_components()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        if components.is_empty() {
            self.status_on_delete(&[]);
            return;
        }
        self.status_on_delete(&components);
        self.run_event(ComponentEv::Delete(components), ctx);
        self.ui.map.selected.clear();
    }
}
