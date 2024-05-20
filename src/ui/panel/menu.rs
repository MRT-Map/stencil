use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContexts};
use bevy_mouse_tracking::MousePosWorld;
#[cfg(debug_assertions)]
use egui_notify::ToastLevel;

#[cfg(debug_assertions)]
use crate::error::log::{ErrorLogEntry, ERROR_LOG};
use crate::{
    action::Action,
    component::actions::undo_redo::UndoRedoAct,
    error::log::OpenErrorLogViewerAct,
    info_windows::InfoWindowsAct,
    keymaps::settings_editor::OpenKeymapSettingsAct,
    project::events::ProjectAct,
    ui::{
        panel::status::Status,
        tilemap::{settings_editor::TileSettingsAct, tile::PendingTiles},
    },
    window::settings_editor::OpenWindowSettingsAct,
};

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
                button!(ui, event_writer, "Save project", ProjectAct::Save);
            });
            egui::menu::menu_button(ui, "Edit", |ui| {
                button!(ui, event_writer, "Undo", UndoRedoAct::Undo);
                button!(ui, event_writer, "Redo", UndoRedoAct::Redo);
            });
            egui::menu::menu_button(ui, "Settings", |ui| {
                button!(ui, event_writer, "Tilemap", TileSettingsAct::Open);
                button!(ui, event_writer, "Window", OpenWindowSettingsAct);
                button!(ui, event_writer, "Keymap", OpenKeymapSettingsAct);
            });
            egui::menu::menu_button(ui, "Debug", |ui| {
                button!(ui, event_writer, "Error Log", OpenErrorLogViewerAct);
                #[cfg(debug_assertions)]
                {
                    if ui.button("Trigger Warning").clicked() {
                        info!(label = "Trigger Warning", "Clicked menu item");
                        let mut error_log = ERROR_LOG.write().unwrap();
                        error_log.pending_errors.push(ErrorLogEntry::new(
                            &"Warning Triggered",
                            ToastLevel::Warning,
                        ));
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
