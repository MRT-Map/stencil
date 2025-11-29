use serde::{Deserialize, Serialize};

use crate::{
    App,
    shortcut::{ShortcutAction, UiButtonWithShortcutExt},
    ui::dock::DockWindow,
};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct HistoryViewerWindow;

impl DockWindow for HistoryViewerWindow {
    fn title(self) -> String {
        "History".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            macro_rules! button {
                ($ui:ident, $label:literal, $action:expr, $f:block) => {
                    if app.menu_button_fn("history viewer menu", $ui, $label, $action) {
                        $f
                    }
                };
            }
            button!(ui, "Undo", Some(ShortcutAction::Undo), {
                app.history_undo(ui.ctx());
            });
            button!(ui, "Redo", Some(ShortcutAction::Redo), {
                app.history_redo(ui.ctx());
            });
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
