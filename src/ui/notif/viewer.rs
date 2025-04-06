use bevy::prelude::*;
use bevy_egui::egui;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ui::{
    notif::NOTIF_LOG,
    panel::dock::{open_dock_window, DockLayout, DockWindow, PanelParams},
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct NotifLogViewer;

#[derive(Clone, Copy, Event)]
pub struct OpenNotifLogViewerEv;

impl DockWindow for NotifLogViewer {
    fn title(self) -> String {
        "Notification Log".into()
    }
    fn ui(self, params: &mut PanelParams, ui: &mut egui::Ui) {
        let PanelParams { .. } = params;
        let Ok(notif_log) = NOTIF_LOG.try_read() else {
            ui.label("Loading...");
            return;
        };
        for entry in notif_log.notifs.iter().rev() {
            ui.colored_label(
                egui::Color32::WHITE,
                format!(
                    "{}",
                    DateTime::<Utc>::from(entry.timestamp).format("%d/%m/%Y %T")
                ),
            );
            ui.colored_label(egui::Color32::YELLOW, &entry.message);
            ui.separator();
        }
        if notif_log.notifs.is_empty() {
            ui.label("No errors, thankfully");
        }
    }
}

pub fn on_log_viewer(_trigger: Trigger<OpenNotifLogViewerEv>, mut state: ResMut<DockLayout>) {
    open_dock_window(&mut state, NotifLogViewer);
}
