use std::{
    fmt::Debug,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Utc};
use egui_notify::{Toast, ToastLevel, Toasts};
use serde::{Deserialize, Serialize};

use crate::{App, settings::misc_settings::MiscSettings, ui::dock::DockWindow};

#[derive(Default)]
pub struct NotifState {
    pub notifs: Vec<Notif>,
    pub toasts: Toasts,
}
impl NotifState {
    pub fn push<S: Into<egui::RichText>>(
        &mut self,
        message: S,
        level: ToastLevel,
        misc_settings: &MiscSettings,
    ) {
        let message = message.into();
        self.toasts
            .add(Toast::custom(message.clone(), level.clone()))
            .duration(
                ((level == ToastLevel::Info || level == ToastLevel::Success)
                    && misc_settings.notif_duration != 0)
                    .then(|| Duration::from_secs(misc_settings.notif_duration)),
            );
        self.notifs.push(Notif::new(message, level));
    }
}
#[derive(Clone, Debug)]
pub struct Notif {
    pub timestamp: SystemTime,
    pub level: ToastLevel,
    pub message: egui::RichText,
}
impl Notif {
    pub fn new<S: Into<egui::RichText>>(message: S, level: ToastLevel) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message: message.into(),
        }
    }
}

impl App {
    pub fn notifs(&mut self, ctx: &egui::Context) {
        self.ui.notifs.toasts.show(ctx);
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct NotifLogWindow;

impl DockWindow for NotifLogWindow {
    fn title(&self) -> String {
        "Notification Log".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        for entry in app.ui.notifs.notifs.iter().rev() {
            ui.colored_label(
                egui::Color32::WHITE,
                format!(
                    "{}",
                    DateTime::<Utc>::from(entry.timestamp).format("%d/%m/%Y %T")
                ),
            );
            ui.colored_label(egui::Color32::YELLOW, entry.message.clone());
            ui.separator();
        }
        if app.ui.notifs.notifs.is_empty() {
            ui.label("No notifications");
        }
    }
}
