use tracing::info;

use crate::{App, map::MapWindow};

impl MapWindow {
    pub fn select_components(app: &mut App, ui: &egui::Ui, response: &egui::Response) {
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

        if ui.ctx().input(|a| a.modifiers.shift) {
            if let Some(pos) = app
                .ui
                .map
                .selected_components
                .iter()
                .position(|a| a == hovered_component)
            {
                info!(id=%hovered_component, "Deselected");
                app.ui.map.selected_components.remove(pos);
            } else {
                info!(id=%hovered_component, "Selected");
                app.ui
                    .map
                    .selected_components
                    .push(hovered_component.to_owned());
            }
        } else {
            info!(id=%hovered_component, "Deselected all and selected one");
            app.ui.map.selected_components.clear();
            app.ui
                .map
                .selected_components
                .push(hovered_component.to_owned());
        }
        app.status_select(ui.ctx());
    }
}
