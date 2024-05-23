use bevy_egui::egui;
use itertools::Itertools;

use crate::{
    action::Action,
    history::HistoryAct,
    ui::panel::dock::{DockWindow, PanelParams, TabViewer},
};

#[derive(Clone, Copy)]
pub struct HistoryViewer;

impl DockWindow for HistoryViewer {
    fn title(self) -> String {
        "History".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams {
            history, actions, ..
        } = tab_viewer.params;
        ui.horizontal(|ui| {
            if ui.button("Undo").clicked() {
                actions.send(Action::new(HistoryAct::Undo));
            }
            if ui.button("Redo").clicked() {
                actions.send(Action::new(HistoryAct::Redo));
            }
        });
        for entry in &history.undo_stack {
            ui.label(entry.iter().map(ToString::to_string).join("; "));
        }
        ui.colored_label(egui::Color32::YELLOW, "Current State");
        for entry in history.redo_stack.iter().rev() {
            ui.label(entry.iter().map(ToString::to_string).join("; "));
        }
    }
    fn closeable(self) -> bool {
        false
    }
}
