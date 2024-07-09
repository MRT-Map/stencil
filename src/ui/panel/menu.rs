use std::collections::HashSet;

use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::egui::scroll_area::ScrollBarVisibility;
use bevy_mouse_tracking::MousePosWorld;
use egui_notify::ToastLevel;

use crate::{
    component::panels::{
        component_editor::OpenComponentEditorAct, component_list::OpenComponentListAct,
    },
    history::{history_viewer::OpenHistoryViewerAct, HistoryAct},
    info_windows::InfoWindowsAct,
    keymaps::settings_editor::{KeymapSettingsEditor, OpenKeymapSettingsAct},
    misc_config::settings_editor::{MiscSettingsEditor, OpenMiscSettingsAct},
    project::{events::ProjectAct, project_editor::OpenProjectEditorAct},
    tile::zoom::Zoom,
    ui::{
        notif::{viewer::OpenNotifLogViewerAct, NotifLogRwLockExt, NOTIF_LOG},
        panel::{
            dock::{DockWindow, DockWindows, PanelDockState, ResetPanelDockStateAct},
            status::Status,
        },
        tilemap::{
            settings_editor::{TileSettingsAct, TileSettingsEditor},
            tile::PendingTiles,
        },
    },
    window::settings_editor::{OpenWindowSettingsAct, WindowSettingsEditor},
};

#[derive(Clone, Copy, Event)]
pub struct OpenAllSettingsAct;

#[allow(clippy::needless_pass_by_value)]
pub fn ui_sy(
    mut ctx: EguiContexts,
    mut commands: Commands,
    diagnostics: Res<DiagnosticsStore>,
    mouse_pos_world: Res<MousePosWorld>,
    pending_tiles: Res<PendingTiles>,
    status: Res<Status>,
    zoom: Res<Zoom>,
) {
    let Some(ctx) = ctx.try_ctx_mut() else {
        return;
    };
    egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            macro_rules! button {
                ($ui:ident, $commands:ident, $label:literal, $action:expr) => {
                    if $ui.button($label).clicked() {
                        info!(label = $label, "Clicked menu item");
                        $commands.trigger($action);
                    }
                };
            }

            #[allow(clippy::cognitive_complexity)]
            egui::menu::menu_button(
                ui,
                format!("Stencil v{}", env!("CARGO_PKG_VERSION")),
                |ui| {
                    button!(ui, commands, "Info", InfoWindowsAct::Info);
                    button!(ui, commands, "Changelog", InfoWindowsAct::Changelog);
                    button!(ui, commands, "Manual", InfoWindowsAct::Manual);
                    button!(ui, commands, "Licenses", InfoWindowsAct::Licenses);
                    ui.separator();
                    button!(ui, commands, "Quit", InfoWindowsAct::Quit(false));
                },
            );
            #[allow(clippy::cognitive_complexity)]
            egui::menu::menu_button(ui, "File", |ui| {
                button!(ui, commands, "Open...", ProjectAct::Open);
                button!(ui, commands, "Reload", ProjectAct::Reload);
                button!(ui, commands, "Save", ProjectAct::Save(false));
            });
            #[allow(clippy::cognitive_complexity)]
            egui::menu::menu_button(ui, "Edit", |ui| {
                button!(ui, commands, "Undo", HistoryAct::Undo);
                button!(ui, commands, "Redo", HistoryAct::Redo);
            });
            #[allow(clippy::cognitive_complexity)]
            egui::menu::menu_button(ui, "View", |ui| {
                button!(ui, commands, "Component List", OpenComponentListAct);
                button!(ui, commands, "Component Editor", OpenComponentEditorAct);
                button!(ui, commands, "Project", OpenProjectEditorAct);
                button!(ui, commands, "History", OpenHistoryViewerAct);
                button!(ui, commands, "Notification Log", OpenNotifLogViewerAct);
                ui.separator();
                button!(ui, commands, "Reset Layout", ResetPanelDockStateAct);
            });
            #[allow(clippy::cognitive_complexity)]
            egui::menu::menu_button(ui, "Settings", |ui| {
                button!(ui, commands, "Open All", OpenAllSettingsAct);
                ui.separator();
                button!(ui, commands, "Tilemap", TileSettingsAct::Open);
                button!(ui, commands, "Window", OpenWindowSettingsAct);
                button!(ui, commands, "Keymap", OpenKeymapSettingsAct);
                button!(ui, commands, "Misc", OpenMiscSettingsAct);
            });
            #[cfg(debug_assertions)]
            {
                #[allow(clippy::cognitive_complexity)]
                egui::menu::menu_button(ui, "Debug", |ui| {
                    if ui.button("Trigger Warning").clicked() {
                        info!(label = "Trigger Warning", "Clicked menu item");
                        NOTIF_LOG.push(&"Warning Triggered", ToastLevel::Warning);
                    }
                    if ui.button("Trigger Panic").clicked() {
                        info!(label = "Trigger Panic", "Clicked menu item");
                        panic!("Panic Triggered");
                    }
                });
            }
            ui.separator();

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
                    "x: {} z: {} \u{1f50d}: {:.2}",
                    mouse_pos_world.round().x as i32,
                    -mouse_pos_world.round().y as i32,
                    zoom.0
                ));
                ui.separator();
                ui.label(format!("# Pending Tiles: {}", pending_tiles.0.len()));
                ui.separator();

                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    egui::ScrollArea::horizontal()
                        .max_width(ui.available_width())
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            ui.label(status.0.to_owned().color(egui::Color32::WHITE));
                        });
                });
            });
        });
    });
}

pub fn on_all_settings(_trigger: Trigger<OpenAllSettingsAct>, mut state: ResMut<PanelDockState>) {
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
