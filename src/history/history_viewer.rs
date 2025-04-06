use bevy::prelude::*;
use bevy_egui::egui;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    history::HistoryEv,
    ui::panel::dock::{open_dock_window, DockLayout, DockWindow, PanelParams},
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct HistoryViewer;

#[derive(Clone, Copy, Event)]
pub struct OpenHistoryViewerEv;

impl DockWindow for HistoryViewer {
    fn title(self) -> String {
        "History".into()
    }
    fn ui(self, params: &mut PanelParams, ui: &mut egui::Ui) {
        let PanelParams {
            history, commands, ..
        } = params;
        ui.horizontal(|ui| {
            if ui.button("Undo").clicked() {
                commands.trigger(HistoryEv::Undo);
            }
            if ui.button("Redo").clicked() {
                commands.trigger(HistoryEv::Redo);
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
}

pub fn on_history_viewer(_trigger: Trigger<OpenHistoryViewerEv>, mut state: ResMut<DockLayout>) {
    open_dock_window(&mut state, HistoryViewer);
}
