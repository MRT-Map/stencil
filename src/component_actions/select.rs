use tracing::info;

use crate::{App, map::MapWindow};

impl MapWindow {
    pub fn select_components(&mut self, app: &App, ui: &egui::Ui, response: &egui::Response) {
        if app.mode.is_editing() {
            self.selected_components.clear();
            return;
        }
        if (!response.clicked_by(egui::PointerButton::Primary)
            && !response.clicked_by(egui::PointerButton::Secondary))
        {
            return;
        }

        let Some(hovered_component) = &self.hovered_component else {
            info!(ids=?self.selected_components, "Deselected all");
            self.selected_components.clear();
            return;
        };

        if ui.ctx().input(|a| a.modifiers.shift) {
            if let Some(pos) = self
                .selected_components
                .iter()
                .position(|a| a == hovered_component)
            {
                info!(id=%hovered_component, "Deselected");
                self.selected_components.remove(pos);
            } else {
                info!(id=%hovered_component, "Selected");
                self.selected_components.push(hovered_component.to_owned());
            }
        } else {
            info!(id=%hovered_component, "Deselected all and selected one");
            self.selected_components.clear();
            self.selected_components.push(hovered_component.to_owned());
        }
    }
}
