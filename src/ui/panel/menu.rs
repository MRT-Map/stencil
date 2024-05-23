use std::collections::HashSet;

use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContexts};
use bevy_mouse_tracking::MousePosWorld;
#[cfg(debug_assertions)]
use egui_notify::ToastLevel;

#[cfg(debug_assertions)]
use crate::notification::{Notif, NOTIF_LOG};
use crate::{
    action::Action,
    history::HistoryAct,
    info_windows::InfoWindowsAct,
    keymaps::settings_editor::{KeymapSettingsEditor, OpenKeymapSettingsAct},
    misc_config::settings_editor::{MiscSettingsEditor, OpenMiscSettingsAct},
    notification::{viewer::OpenNotifLogViewerAct, NotifLogRwLockExt},
    project::events::ProjectAct,
    ui::{
        panel::{
            dock::{DockWindow, DockWindows, PanelDockState},
            status::Status,
        },
        tilemap::{
            settings_editor::{TileSettingsAct, TileSettingsEditor},
            tile::PendingTiles,
        },
    },
    window::settings_editor::{OpenWindowSettingsAct, WindowSettingsEditor},
};

pub struct OpenAllSettingsAct;

#[allow(clippy::needless_pass_by_value)]
pub fn ui_sy(
    mut ctx: EguiContexts,
    mut event_writer: EventWriter<Action>,
    diagnostics: Res<DiagnosticsStore>,
    mouse_pos_world: Res<MousePosWorld>,
    pending_tiles: Res<PendingTiles>,
    status: Res<Status>,
) {
    let Some(ctx) = ctx.try_ctx_mut() else {
        return;
    };
    egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            macro_rules! button {
                ($ui:ident, $ew:ident, $label:literal, $action:expr) => {
                    if $ui.button($label).clicked() {
                        info!(label = $label, "Clicked menu item");
                        $ew.send(Action::new($action));
                    }
                };
            }

            #[allow(clippy::cognitive_complexity)]
            egui::menu::menu_button(
                ui,
                format!("Stencil v{}", env!("CARGO_PKG_VERSION")),
                |ui| {
                    button!(ui, event_writer, "Info", InfoWindowsAct::Info);
                    button!(ui, event_writer, "Changelog", InfoWindowsAct::Changelog);
                    button!(ui, event_writer, "Manual", InfoWindowsAct::Manual);
                    button!(ui, event_writer, "Licenses", InfoWindowsAct::Licenses);
                    ui.separator();
                    button!(ui, event_writer, "Quit", InfoWindowsAct::Quit(false));
                },
            );
            egui::menu::menu_button(ui, "File", |ui| {
                button!(
                    ui,
                    event_writer,
                    "Select project folder",
                    ProjectAct::SelectFolder
                );
                button!(ui, event_writer, "Save project", ProjectAct::Save(false));
            });
            egui::menu::menu_button(ui, "Edit", |ui| {
                button!(ui, event_writer, "Undo", HistoryAct::Undo);
                button!(ui, event_writer, "Redo", HistoryAct::Redo);
            });
            egui::menu::menu_button(ui, "Settings", |ui| {
                button!(ui, event_writer, "Open All", OpenAllSettingsAct);
                ui.separator();
                button!(ui, event_writer, "Tilemap", TileSettingsAct::Open);
                button!(ui, event_writer, "Window", OpenWindowSettingsAct);
                button!(ui, event_writer, "Keymap", OpenKeymapSettingsAct);
                button!(ui, event_writer, "Misc", OpenMiscSettingsAct);
            });
            egui::menu::menu_button(ui, "Debug", |ui| {
                button!(ui, event_writer, "Notification Log", OpenNotifLogViewerAct);
                #[cfg(debug_assertions)]
                {
                    if ui.button("Trigger Warning").clicked() {
                        info!(label = "Trigger Warning", "Clicked menu item");
                        NOTIF_LOG.push(&"Warning Triggered", ToastLevel::Warning);
                    }
                    if ui.button("Trigger Panic").clicked() {
                        info!(label = "Trigger Panic", "Clicked menu item");
                        panic!("Panic Triggered");
                    }
                }
            });
            ui.separator();
            ui.label(status.0.to_owned().color(egui::Color32::WHITE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                ui.label(format!(
                    "FPS: {} ({})",
                    diagnostics
                        .get(&FrameTimeDiagnosticsPlugin::FPS)
                        .and_then(Diagnostic::average)
                        .map_or_else(|| "???".into(), |fps| format!("{fps:.2}")),
                    diagnostics
                        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                        .and_then(Diagnostic::average)
                        .map_or_else(|| "???".into(), |ft| format!("{ft:.2}ms")),
                ));
                ui.separator();
                ui.label(format!(
                    "x: {} z: {}",
                    mouse_pos_world.round().x as i32,
                    -mouse_pos_world.round().y as i32
                ));
                ui.separator();
                ui.label(format!("# Pending Tiles: {}", pending_tiles.0.len()));
            })
        });
    });
}

pub fn all_settings_asy(mut actions: EventReader<Action>, mut state: ResMut<PanelDockState>) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(OpenAllSettingsAct))
            && !state
                .state
                .iter_all_tabs()
                .any(|(_, a)| a.title() == MiscSettingsEditor.title())
        {
            let all_tabs = state
                .state
                .iter_all_tabs()
                .map(|(_, a)| a.title())
                .collect::<HashSet<_>>();
            let settings_tabs = [
                TileSettingsEditor.into(),
                WindowSettingsEditor.into(),
                KeymapSettingsEditor.into(),
                MiscSettingsEditor.into(),
            ]
            .into_iter()
            .filter(|a: &DockWindows| !all_tabs.contains(&a.title()))
            .collect::<Vec<_>>();
            if settings_tabs.is_empty() {
                NOTIF_LOG.push(&"All settings tabs are already open", ToastLevel::Info);
            } else {
                state.state.add_window(settings_tabs);
            }
        }
    }
}
