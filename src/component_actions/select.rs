use tracing::info;

use crate::{App, map::MapWindow, project::pla3::FullId};

impl MapWindow {
    pub fn select_hovered_component(app: &mut App, ui: &egui::Ui, response: &egui::Response) {
        if app.mode.is_editing() {
            app.ui.map.selected_components.clear();
            return;
        }
        if !response.clicked_by(egui::PointerButton::Primary)
            && !response.clicked_by(egui::PointerButton::Secondary)
        {
            return;
        }

        let Some(hovered_component) = &app.ui.map.hovered_component else {
            info!(ids=?app.ui.map.selected_components, "Deselected all");
            app.ui.map.selected_components.clear();
            app.status_default(ui.ctx());
            return;
        };
        app.select_component(ui, hovered_component.to_owned());
    }
}
impl App {
    pub fn select_component(&mut self, ui: &egui::Ui, id: FullId) {
        if ui.ctx().input(|a| a.modifiers.shift) {
            if let Some(pos) = self
                .ui
                .map
                .selected_components
                .iter()
                .position(|a| *a == id)
            {
                info!(%id, "Deselected");
                self.ui.map.selected_components.remove(pos);
            } else {
                info!(%id, "Selected");
                self.ui.map.selected_components.push(id);
            }
        } else {
            info!(%id, "Deselected all and selected one");
            self.ui.map.selected_components.clear();
            self.ui.map.selected_components.push(id);
        }
        self.status_select(ui.ctx());
    }
}
