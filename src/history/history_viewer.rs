use bevy_egui::egui;

use crate::ui::panel::dock::{DockWindow, PanelParams, TabViewer};

#[derive(Clone, Copy)]
pub struct HistoryViewer;

impl DockWindow for HistoryViewer {
    fn title(self) -> String {
        "History".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams { history, .. } = tab_viewer.params;
        for entry in &history.undo_stack {
            ui.label(format!("{entry:?}"));
        }
        ui.colored_label(egui::Color32::YELLOW, "Current State");
        for entry in history.redo_stack.iter().rev() {
            ui.label(format!("{entry:?}"));
        }
    }
    fn closeable(self) -> bool {
        false
    }
}
