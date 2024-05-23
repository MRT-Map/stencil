use bevy::prelude::{EventReader, ResMut};
use bevy_egui::egui;
use chrono::{DateTime, Utc};

use crate::{
    action::Action,
    ui::{
        notif::NOTIF_LOG,
        panel::dock::{window_action_handler, DockWindow, PanelDockState, PanelParams, TabViewer},
    },
};

#[derive(Clone, Copy)]
pub struct NotifLogViewer;

pub struct OpenNotifLogViewerAct;

impl DockWindow for NotifLogViewer {
    fn title(self) -> String {
        "Notification Log".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams { .. } = tab_viewer.params;
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

pub fn log_viewer_asy(mut state: ResMut<PanelDockState>, mut actions: EventReader<Action>) {
    for event in actions.read() {
        window_action_handler(event, &mut state, OpenNotifLogViewerAct, NotifLogViewer);
    }
}
