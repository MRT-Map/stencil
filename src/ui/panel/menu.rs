use std::collections::HashSet;

use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, egui::scroll_area::ScrollBarVisibility, EguiContexts};
use egui_notify::ToastLevel;

#[cfg(debug_assertions)]
use crate::inspector::ShowInspector;
use crate::{
    component::panels::{
        component_editor::OpenComponentEditorEv, component_list::OpenComponentListEv,
    },
    history::{history_viewer::OpenHistoryViewerEv, HistoryEv},
    info_windows::InfoWindowsEv,
    keymaps::settings_editor::{KeymapSettingsEditor, OpenKeymapSettingsEv},
    misc_config::settings_editor::{MiscSettingsEditor, OpenMiscSettingsEv},
    project::{events::ProjectEv, project_editor::OpenProjectEditorEv},
    ui::{
        map::settings_editor::{TileSettingsEditor, TileSettingsEv},
        notif::{viewer::OpenNotifLogViewerEv, NotifLogRwLockExt, NOTIF_LOG},
        panel::{
            dock::{DockLayout, DockWindow, DockWindows, ResetPanelDockStateEv},
            status::Status,
        },
    },
    window::settings_editor::{OpenWindowSettingsEv, WindowSettingsEditor},
};

#[derive(Clone, Copy, Event)]
pub struct OpenAllSettingsEv;

#[expect(clippy::needless_pass_by_value)]
pub fn ui_sy(
    mut ctx: EguiContexts,
    mut commands: Commands,
    diagnostics: Res<DiagnosticsStore>,
    status: Res<Status>,
    #[cfg(debug_assertions)] inspector: Option<Res<ShowInspector>>,
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

            #[expect(clippy::cognitive_complexity)]
            egui::menu::menu_button(
                ui,
                format!("Stencil v{}", env!("CARGO_PKG_VERSION")),
                |ui| {
                    button!(ui, commands, "Info", InfoWindowsEv::Info);
                    button!(ui, commands, "Changelog", InfoWindowsEv::Changelog);
                    button!(ui, commands, "Manual", InfoWindowsEv::Manual);
                    button!(ui, commands, "Licenses", InfoWindowsEv::Licenses);
                    ui.separator();
                    button!(ui, commands, "Quit", InfoWindowsEv::Quit(false));
                },
            );
            egui::menu::menu_button(ui, "File", |ui| {
                button!(ui, commands, "Open...", ProjectEv::Open);
                button!(ui, commands, "Reload", ProjectEv::Reload);
                button!(ui, commands, "Save", ProjectEv::Save(false));
            });
            egui::menu::menu_button(ui, "Edit", |ui| {
                button!(ui, commands, "Undo", HistoryEv::Undo);
                button!(ui, commands, "Redo", HistoryEv::Redo);
            });
            #[expect(clippy::cognitive_complexity)]
            egui::menu::menu_button(ui, "View", |ui| {
                button!(ui, commands, "Component List", OpenComponentListEv);
                button!(ui, commands, "Component Editor", OpenComponentEditorEv);
                button!(ui, commands, "Project", OpenProjectEditorEv);
                button!(ui, commands, "History", OpenHistoryViewerEv);
                button!(ui, commands, "Notification Log", OpenNotifLogViewerEv);
                ui.separator();
                button!(ui, commands, "Reset Layout", ResetPanelDockStateEv);
            });
            #[expect(clippy::cognitive_complexity)]
            egui::menu::menu_button(ui, "Settings", |ui| {
                button!(ui, commands, "Open All", OpenAllSettingsEv);
                ui.separator();
                button!(ui, commands, "Tilemap", TileSettingsEv::Open);
                button!(ui, commands, "Window", OpenWindowSettingsEv);
                button!(ui, commands, "Keymap", OpenKeymapSettingsEv);
                button!(ui, commands, "Misc", OpenMiscSettingsEv);
            });
            #[cfg(debug_assertions)]
            {
                egui::menu::menu_button(ui, "Debug", |ui| {
                    if ui.button("Trigger Warning").clicked() {
                        info!(label = "Trigger Warning", "Clicked menu item");
                        NOTIF_LOG.push("Warning Triggered", ToastLevel::Warning);
                    }
                    if ui.button("Trigger Panic").clicked() {
                        info!(label = "Trigger Panic", "Clicked menu item");
                        panic!("Panic Triggered");
                    }
                    if ui.button("Show Inspector").clicked() {
                        if inspector.is_some() {
                            commands.remove_resource::<ShowInspector>();
                        } else {
                            commands.init_resource::<ShowInspector>();
                        }
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

                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    egui::ScrollArea::horizontal()
                        .max_width(ui.available_width())
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            ui.label(status.0.clone().color(egui::Color32::WHITE));
                        });
                });
            });
        });
    });
}

pub fn on_all_settings(_trigger: Trigger<OpenAllSettingsEv>, mut state: ResMut<DockLayout>) {
    let all_tabs = state
        .0
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
        NOTIF_LOG.push("All settings tabs are already open", ToastLevel::Info);
    } else {
        state.0.add_window(settings_tabs);
    }
}
