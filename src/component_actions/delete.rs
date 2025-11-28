use crate::{App, component_actions::event::ComponentEv};

impl App {
    pub fn delete_selected_components(&mut self, ctx: &egui::Context) {
        let components = self
            .ui
            .dock_layout
            .map_window()
            .selected_components(&self.project.components)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        if components.is_empty() {
            return;
        }
        self.run_event(ComponentEv::Delete(components), ctx);
        self.ui
            .dock_layout
            .map_window_mut()
            .selected_components
            .clear();
    }
}
