use std::{fmt::Debug, sync::RwLock, time::SystemTime};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use chrono::{DateTime, Utc};
use egui_notify::{Toast, ToastLevel, Toasts};
use once_cell::sync::Lazy;

use crate::{
    misc::Action,
    ui::panel::dock::{DockWindow, PanelDockState, PanelParams, TabViewer},
};

pub static ERROR_LOG: Lazy<RwLock<ErrorLog>> = Lazy::new(|| RwLock::new(ErrorLog::default()));

#[derive(Default, Resource)]
pub struct NotifToasts(pub Toasts);

#[derive(Clone, Debug, Default)]
pub struct ErrorLog {
    pub errors: Vec<ErrorLogEntry>,
    pub pending_errors: Vec<ErrorLogEntry>,
}

#[derive(Clone, Debug)]
pub struct ErrorLogEntry {
    pub timestamp: SystemTime,
    pub level: ToastLevel,
    pub message: String,
}
impl ErrorLogEntry {
    pub fn new<S: ToString>(message: &S, level: ToastLevel) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message: message.to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ErrorLogViewer;

pub struct OpenErrorLogViewerAct;

impl DockWindow for ErrorLogViewer {
    fn title(self) -> String {
        "Errors".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams { .. } = tab_viewer.params;
        let Ok(error_log) = ERROR_LOG.try_read() else {
            ui.label("Loading...");
            return;
        };
        for entry in error_log.errors.iter().rev() {
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
        if error_log.errors.is_empty() {
            ui.label("No errors, thankfully");
        }
    }
}

pub fn update_error_log_sy(
    mut state: ResMut<PanelDockState>,
    mut toasts: ResMut<NotifToasts>,
    mut ctx: EguiContexts,
    mut actions: EventReader<Action>,
) {
    let Ok(mut error_log) = ERROR_LOG.try_write() else {
        return;
    };

    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(OpenErrorLogViewerAct)) {
            let tab = state
                .state
                .iter_all_tabs()
                .find(|(_, a)| a.title() == ErrorLogViewer.title())
                .map(|a| a.0);
            if let Some(tab) = tab {
                state.state.set_focused_node_and_surface(tab);
            } else {
                state.state.add_window(vec![ErrorLogViewer.into()]);
            }
        }
    }
    if let Some(ctx) = ctx.try_ctx_mut() {
        toasts.0.show(ctx);
    };

    if error_log.pending_errors.is_empty() {
        return;
    }
    let pending_errors = error_log.pending_errors.to_owned();
    for error in pending_errors {
        toasts
            .0
            .add(Toast::custom(&error.message, error.level.to_owned()))
            .set_duration(None);
        error_log.errors.push(error);
    }
    error_log.pending_errors.clear();
}

pub trait AddToErrorLog<T: Default> {
    #[must_use]
    fn add_to_error_log(self, level: ToastLevel) -> Self;
    #[must_use]
    fn unwrap_or_default_and_log(self, level: ToastLevel) -> T;
}

impl<T: Default, E: ToString + Debug> AddToErrorLog<T> for Result<T, E> {
    fn add_to_error_log(self, level: ToastLevel) -> Self {
        self.inspect_err(|e| {
            let mut error_log = ERROR_LOG.write().unwrap();
            error_log
                .pending_errors
                .push(ErrorLogEntry::new(&e.to_string(), level));
        })
    }
    fn unwrap_or_default_and_log(self, level: ToastLevel) -> T {
        self.add_to_error_log(level).unwrap_or_default()
    }
}
