use std::{fmt::Debug, sync::RwLock, time::SystemTime};

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui_notify::{Toast, ToastLevel, Toasts};
use once_cell::sync::Lazy;
use viewer::NotifLogViewer;

use crate::{
    action::Action,
    ui::panel::dock::{DockWindow, PanelDockState},
};

pub mod viewer;

pub static NOTIF_LOG: Lazy<RwLock<NotifLog>> = Lazy::new(|| RwLock::new(NotifLog::default()));

#[derive(Default, Resource)]
pub struct NotifToasts(pub Toasts);

#[derive(Clone, Debug, Default)]
pub struct NotifLog {
    pub notifs: Vec<Notif>,
    pub pending_notifs: Vec<Notif>,
}
impl NotifLog {
    pub fn push<S: ToString>(&mut self, message: &S, level: ToastLevel) {
        self.pending_notifs.push(Notif::new(message, level))
    }
}

#[derive(Clone, Debug)]
pub struct Notif {
    pub timestamp: SystemTime,
    pub level: ToastLevel,
    pub message: String,
}
impl Notif {
    pub fn new<S: ToString>(message: &S, level: ToastLevel) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message: message.to_string(),
        }
    }
}

pub fn update_notifs_asy(mut toasts: ResMut<NotifToasts>, mut ctx: EguiContexts) {
    let Ok(mut notif_log) = NOTIF_LOG.try_write() else {
        return;
    };

    if let Some(ctx) = ctx.try_ctx_mut() {
        toasts.0.show(ctx);
    };

    if notif_log.pending_notifs.is_empty() {
        return;
    }
    let pending_notifs = notif_log.pending_notifs.to_owned();
    for notif in pending_notifs {
        toasts
            .0
            .add(Toast::custom(&notif.message, notif.level.to_owned()))
            .set_duration(None);
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
            let mut notif_log = NOTIF_LOG.write().unwrap();
            notif_log
                .pending_notifs
                .push(Notif::new(&e.to_string(), level));
        })
    }
    fn unwrap_or_default_and_notif(self, level: ToastLevel) -> T {
        self.notif_error(level).unwrap_or_default()
    }
}

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotifToasts>()
            .add_systems(Update, update_notifs_asy);
    }
}
