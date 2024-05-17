use std::{fmt::Debug, sync::RwLock, time::SystemTime};

use bevy::prelude::ResMut;
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use once_cell::sync::Lazy;
use tracing::log::Level::Error;

use crate::{
    misc::Action,
    project::project_editor::ProjectAct,
    ui::panel::dock::{DockWindow, PanelDockState, PanelParams, TabViewer},
    window::settings_editor::WindowSettingsEditor,
};

pub static ERROR_LOG: Lazy<RwLock<ErrorLog>> = Lazy::new(|| RwLock::new(ErrorLog::default()));

#[derive(Clone, Debug, Default)]
pub struct ErrorLog {
    pub errors: Vec<ErrorLogEntry>,
    pub last_length: usize,
}

#[derive(Clone, Debug)]
pub struct ErrorLogEntry {
    pub timestamp: SystemTime,
    pub count: usize,
    pub message: String,
}

#[derive(Clone, Copy)]
pub struct ErrorLogViewer;

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
            ui.colored_label(egui::Color32::WHITE, {
                let mut s = format!("{:?}", entry.timestamp);
                if entry.count != 1 {
                    s.push_str(&format!(" ({})", entry.count));
                }
                s
            });
            ui.colored_label(egui::Color32::YELLOW, &entry.message);
            ui.separator();
        }
        if error_log.errors.is_empty() {
            ui.label("No errors, thankfully");
        }
    }
}

pub fn update_error_log_sy(mut state: ResMut<PanelDockState>) {
    let Ok(mut error_log) = ERROR_LOG.try_write() else {
        return;
    };
    error_log.errors.push(ErrorLogEntry {
        timestamp: SystemTime::now(),
        count: 1,
        message: "a".into(),
    });
    if error_log.errors.len() == error_log.last_length {
        return;
    }
    error_log.last_length = error_log.errors.len();
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

pub trait AddToErrorLog<T: Default> {
    #[must_use]
    fn add_to_error_log(self) -> Self;
    #[must_use]
    fn unwrap_or_default_and_log(self) -> T;
}

impl<T: Default, E: ToString + Debug> AddToErrorLog<T> for Result<T, E> {
    fn add_to_error_log(self) -> Self {
        self.inspect_err(|e| {
            let mut error_log = ERROR_LOG.write().unwrap();
            error_log.errors.push(ErrorLogEntry {
                timestamp: SystemTime::now(),
                count: 1,
                message: e.to_string(),
            });
        })
    }
    fn unwrap_or_default_and_log(self) -> T {
        self.add_to_error_log().unwrap_or_default()
    }
}
