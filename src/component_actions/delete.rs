use crate::{App, component_actions::event::ComponentEv};

impl App {
    pub fn delete_selected_components(&mut self, ctx: &egui::Context) {
        let components = self
            .map_selected_components()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        if components.is_empty() {
            self.status_on_delete(&[], ctx);
            return;
        }
        self.status_on_delete(&components, ctx);
        self.run_event(ComponentEv::Delete(components), ctx);
        self.ui.map.selected_components.clear();
    }
}
