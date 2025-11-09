use std::sync::Arc;

use itertools::Itertools;

use crate::{App, map::MapWindow};

impl MapWindow {
    pub fn select_components(&mut self, app: &App, ui: &egui::Ui, response: &egui::Response) {
        if !response.clicked_by(egui::PointerButton::Primary) {
            return;
        }

        let Some(hovered_component) = &self.hovered_component else {
            self.selected_components.clear();
            return;
        };

        if ui.ctx().input(|a| a.modifiers.shift) {
            if let Some(pos) = self
                .selected_components
                .iter()
                .position(|a| a == hovered_component)
            {
                self.selected_components.remove(pos);
            } else {
                self.selected_components.push(hovered_component.to_owned());
            }
        } else {
            self.selected_components.clear();
            self.selected_components.push(hovered_component.to_owned());
        }
    }
}
