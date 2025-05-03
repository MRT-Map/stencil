use std::{
    fmt::Debug,
    sync::{LazyLock, RwLock},
    time::{Duration, SystemTime},
};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_notify::{Toast, ToastLevel, Toasts};

use crate::misc_config::settings::MiscSettings;

pub mod viewer;

pub static NOTIF_LOG: LazyLock<RwLock<NotifLog>> =
    LazyLock::new(|| RwLock::new(NotifLog::default()));

#[derive(Default, Resource)]
pub struct NotifToasts(pub Toasts);

#[derive(Clone, Debug, Default)]
pub struct NotifLog {
    pub notifs: Vec<Notif>,
    pub pending_notifs: Vec<Notif>,
}
impl NotifLog {
    pub fn push<S: Into<egui::RichText>>(&mut self, message: S, level: ToastLevel) {
        self.pending_notifs.push(Notif::new(message, level));
    }
}

pub trait NotifLogRwLockExt {
    fn push<S: Into<egui::RichText>>(&self, message: S, level: ToastLevel);
}
impl NotifLogRwLockExt for RwLock<NotifLog> {
    fn push<S: Into<egui::RichText>>(&self, message: S, level: ToastLevel) {
        let mut notif_log = self.write().unwrap();
        notif_log.push(message, level);
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

#[expect(clippy::needless_pass_by_value)]
pub fn update_notifs_sy(
    mut toasts: ResMut<NotifToasts>,
    mut ctx: EguiContexts,
    misc_settings: Res<MiscSettings>,
) {
    let Ok(mut notif_log) = NOTIF_LOG.try_write() else {
        return;
    };

    if let Some(ctx) = ctx.try_ctx_mut() {
        toasts.0.show(ctx);
    }

    if notif_log.pending_notifs.is_empty() {
        return;
    }
    let pending_notifs = notif_log.pending_notifs.clone();
    for notif in pending_notifs {
        toasts
            .0
            .add(Toast::custom(notif.message.to_owned(), notif.level.clone()))
            .duration(
                ((notif.level == ToastLevel::Info || notif.level == ToastLevel::Success)
                    && misc_settings.notif_duration != 0)
                    .then(|| Duration::from_secs(misc_settings.notif_duration)),
            );
        notif_log.notifs.push(notif);
    }
    notif_log.pending_notifs.clear();
}

pub trait AddToErrorLog<T: Default> {
    #[must_use]
    fn notif_error(self, level: ToastLevel) -> Self;
    #[must_use]
    fn unwrap_or_default_and_notif(self, level: ToastLevel) -> T;
}

impl<T: Default, E: ToString + Debug> AddToErrorLog<T> for Result<T, E> {
    fn notif_error(self, level: ToastLevel) -> Self {
        self.inspect_err(|e| {
            NOTIF_LOG.push(e.to_string(), level);
        })
    }
    fn unwrap_or_default_and_notif(self, level: ToastLevel) -> T {
        self.notif_error(level).unwrap_or_default()
    }
}

pub struct NotifPlugin;

impl Plugin for NotifPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotifToasts>()
            .add_systems(Update, update_notifs_sy)
            .add_observer(viewer::on_log_viewer);
    }
}
