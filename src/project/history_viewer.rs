use serde::{Deserialize, Serialize};

use crate::{
    App,
    shortcut::{ShortcutAction, UiButtonWithShortcutExt},
    ui::dock::DockWindow,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HistoryViewerWindow;

impl DockWindow for HistoryViewerWindow {
    fn title(&self) -> String {
        "History".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            if ui
                .button_with_shortcut("Undo", ShortcutAction::Undo, &mut app.shortcut_settings)
                .clicked()
            {
                app.undo(ui.ctx());
            }
            if ui
                .button_with_shortcut("Redo", ShortcutAction::Redo, &mut app.shortcut_settings)
                .clicked()
            {
                app.redo(ui.ctx());
            }
        });
        ui.separator();

        for entry in &app.project.history.undo_stack {
            ui.label(format!("{entry}"));
        }
        ui.colored_label(egui::Color32::YELLOW, "Current State");
        for entry in app.project.history.redo_stack.iter().rev() {
            ui.label(format!("{entry}"));
        }
    }
}
