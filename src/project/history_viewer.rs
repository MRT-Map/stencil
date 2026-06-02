use serde::{Deserialize, Serialize};

use crate::{App, shortcut::ShortcutAction, ui::dock::DockWindow};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct HistoryViewerWindow;

impl DockWindow for HistoryViewerWindow {
    fn title(self) -> String {
        "History".into()
    }
    #[tracing::instrument(skip_all)]
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            macro_rules! button {
                ($ui:ident, $label:literal, $action:expr) => {
                    if app.menu_button_fn("history viewer menu", $ui, $label, Some($action)) {
                        app.run_action($ui, $action, None);
                    }
                };
            }
            button!(ui, "Undo", ShortcutAction::Undo);
            button!(ui, "Redo", ShortcutAction::Redo);
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
