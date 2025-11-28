use std::{
    fmt::{Debug, Display},
    sync::{
        LazyLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Utc};
use egui_notify::{Anchor, Toast, ToastLevel, Toasts};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{App, settings::misc_settings::MiscSettings, ui::dock::DockWindow};

pub static NOTIF_DURATION: LazyLock<AtomicU64> =
    LazyLock::new(|| AtomicU64::new(MiscSettings::default().notif_duration));

pub struct NotifState {
    pub notifs: Vec<Notif>,
    pub toasts: Toasts,
}

impl Default for NotifState {
    fn default() -> Self {
        Self {
            notifs: Vec::default(),
            toasts: Toasts::default().with_anchor(Anchor::BottomRight),
        }
    }
}

impl NotifState {
    fn push_base(&mut self, message: egui::RichText, level: ToastLevel) {
        let notif_duration = NOTIF_DURATION.load(Ordering::Relaxed);
        self.toasts
            .add(Toast::custom(message.clone(), level.clone()))
            .duration(
                ((level == ToastLevel::Info || level == ToastLevel::Success)
                    && notif_duration != 0)
                    .then(|| Duration::from_secs(notif_duration)),
            );
        self.notifs.push(Notif::new(message, level));
    }

    pub fn push<S: Into<egui::RichText>>(&mut self, message: S, level: ToastLevel) {
        let message = message.into();
        match level {
            ToastLevel::Error => error!(msg = message.text(), "Sending notification"),
            ToastLevel::Warning => warn!(msg = message.text(), "Sending notification"),
            _ => info!(msg = message.text(), "Sending notification"),
        }
        self.push_base(message, level);
    }
    pub fn push_error<S: Display, E: Debug + Display>(
        &mut self,
        message: S,
        error: E,
        level: ToastLevel,
    ) {
        match level {
            ToastLevel::Error => error!(msg=%message, ?error, "Sending notification"),
            ToastLevel::Warning => warn!(msg=%message, ?error, "Sending notification"),
            _ => info!(msg=%message, ?error, "Sending notification"),
        }
        self.push_base(format!("{message}\n{error}").into(), level);
    }
    pub fn push_errors<S: Display, E: Debug + Display>(
        &mut self,
        message: S,
        errors: &[E],
        level: ToastLevel,
    ) {
        match level {
            ToastLevel::Error => error!(msg=%message, ?errors, "Sending notification"),
            ToastLevel::Warning => warn!(msg=%message, ?errors, "Sending notification"),
            _ => info!(msg=%message, ?errors, "Sending notification"),
        }
        self.push_base(
            format!(
                "{message}\n{}",
                errors.iter().map(|e| format!("{e}")).join("\n")
            )
            .into(),
            level,
        );
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
    fn title(self) -> String {
        "Notification Log".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        for entry in app.ui.notifs.notifs.iter().rev() {
            let (colour, notif_type) = match &entry.level {
                ToastLevel::Info => (egui::Color32::WHITE, "Info"),
                ToastLevel::Warning => (egui::Color32::ORANGE, "Warning"),
                ToastLevel::Error => (egui::Color32::RED, "Error"),
                ToastLevel::Success => (egui::Color32::GREEN, "Success"),
                ToastLevel::None => (egui::Color32::GRAY, "None"),
                ToastLevel::Custom(notif_type, colour) => (*colour, &**notif_type),
            };
            ui.horizontal(|ui| {
                ui.colored_label(colour, notif_type);
                ui.separator();
                ui.colored_label(
                    egui::Color32::LIGHT_GRAY,
                    format!(
                        "{}",
                        DateTime::<Utc>::from(entry.timestamp).format("%d/%m/%Y %T")
                    ),
                );
            });
            ui.colored_label(colour, entry.message.clone());
            ui.separator();
        }
        if app.ui.notifs.notifs.is_empty() {
            ui.label("No notifications");
        }
    }
}
