use std::{
    fmt::Debug,
    sync::{LazyLock, atomic::AtomicU64},
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Utc};
use crossbeam_channel::TryRecvError;
use egui_notify::{Anchor, Toast, ToastLevel, Toasts};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{App, settings::misc_settings::MiscSettings, ui::dock::DockWindow};

pub static NOTIF_DURATION: LazyLock<AtomicU64> =
    LazyLock::new(|| AtomicU64::new(MiscSettings::default().notif_duration));

pub static CHANNEL: LazyLock<(
    crossbeam_channel::Sender<Notif>,
    crossbeam_channel::Receiver<Notif>,
)> = LazyLock::new(crossbeam_channel::unbounded);

#[macro_export]
macro_rules! notif {
    (info $message:expr) => {
        ::tracing::info!("{}", $message);
        $crate::ui::notif::NotifState::send(egui_notify::ToastLevel::Info, $message);
    };
    (info $message:expr, $tracing:tt) => {
        ::tracing::info!($tracing);
        $crate::notif!(info $message);
    };
    (success $message:expr) => {
        ::tracing::info!("{}", $message);
        $crate::ui::notif::NotifState::send(egui_notify::ToastLevel::Success, $message);
    };
    (success $message:expr, $tracing:tt) => {
        ::tracing::info!($tracing);
        $crate::notif!(success $message);
    };
    (warning $message:expr) => {
        ::tracing::warn!("{}", $message);
        $crate::ui::notif::NotifState::send(egui_notify::ToastLevel::Warning, $message);
    };
    (warning $message:expr, errors $errors:expr) => {
        use ::itertools::Itertools;
        ::tracing::warn!(errors = ?$errors, "{}", $message);
        $crate::notif!(warning format!("{}\n{}", $message, $errors.iter().map(ToString::to_string).join("\n")));
    };
    (warning $message:expr, errors $errors:expr, $tracing:tt) => {
        use ::itertools::Itertools;
        ::tracing::warn!(errors = ?$errors, $tracing);
        $crate::notif!(warning format!("{}\n{}", $message, $errors.iter().map(ToString::to_string).join("\n")));
    };
    (warning $message:expr, $tracing:tt) => {
        ::tracing::warn!($tracing);
        $crate::notif!(warning $message);
    };
    (error $message:expr) => {
        ::tracing::error!("{}", $message);
        $crate::ui::notif::NotifState::send(egui_notify::ToastLevel::Error, $message);
    };
    (error $message:expr, errors $errors:expr) => {
        use ::itertools::Itertools;
        ::tracing::error!(errors = ?$errors, "{}", $message);
        $crate::notif!(error format!("{}\n{}", $message, $errors.iter().map(ToString::to_string).join("\n")));
    };
    (error $message:expr, errors $errors:expr, $tracing:tt) => {
        use ::itertools::Itertools;
        ::tracing::error!(errors = ?$errors, $tracing);
        $crate::notif!(error format!("{}\n{}", $message, $errors.iter().map(ToString::to_string).join("\n")));
    };
    (error $message:expr, $tracing:tt) => {
        ::tracing::error!($tracing);
        $crate::notif!(error $message);
    };
}

#[derive(Clone, Debug)]
pub struct Notif {
    pub timestamp: SystemTime,
    pub level: ToastLevel,
    pub message: egui::RichText,
}
impl Notif {
    pub fn new<M: Into<egui::RichText>>(level: ToastLevel, message: M) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message: message.into(),
        }
    }
}

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
    #[tracing::instrument(skip_all)]
    pub fn send<M: Into<egui::RichText>>(level: ToastLevel, message: M) {
        let message = message.into();
        info!(message = %message.text(), ?level, "Sending notification");
        CHANNEL
            .0
            .send(Notif::new(level, message))
            .unwrap_or_else(|e| unreachable!("Disconnected channel: {e:#?}"));
    }
    pub fn process_notifs(&mut self, misc_settings: &MiscSettings) {
        loop {
            let notif = match CHANNEL.1.try_recv() {
                Ok(notif) => notif,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => unreachable!("Disconnected channel"),
            };
            self.toasts
                .add(Toast::custom(notif.message.clone(), notif.level.clone()))
                .duration(
                    ((notif.level == ToastLevel::Info || notif.level == ToastLevel::Success)
                        && misc_settings.notif_duration != 0)
                        .then(|| Duration::from_secs(misc_settings.notif_duration)),
                );
            self.notifs.push(notif);
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
    #[tracing::instrument(skip_all)]
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
