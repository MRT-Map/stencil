use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui,
    egui::{Align, Color32},
    EguiContexts,
};
use bevy_mouse_tracking::MousePosWorld;

use crate::{
    component_actions::undo_redo::UndoRedoAct,
    info_windows::InfoWindowsAct,
    load_save::LoadSaveAct,
    misc::Action,
    ui::{
        panel::status::Status,
        tilemap::{settings_editor::TileSettingsAct, tile::PendingTiles},
    },
    window_settings::settings_editor::OpenWindowSettingsAct,
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
    egui::TopBottomPanel::top("menu").show(ctx.ctx_mut(), |ui| {
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
                button!(ui, event_writer, "Load namespace", LoadSaveAct::Load);
                button!(ui, event_writer, "Save namespaces", LoadSaveAct::Save);
            });
            egui::menu::menu_button(ui, "Edit", |ui| {
                button!(ui, event_writer, "Undo", UndoRedoAct::Undo);
                button!(ui, event_writer, "Redo", UndoRedoAct::Redo);
            });
            egui::menu::menu_button(ui, "Settings", |ui| {
                button!(ui, event_writer, "Tilemap", TileSettingsAct::Open);
                button!(ui, event_writer, "Window", OpenWindowSettingsAct);
            });
            ui.separator();
            ui.label(status.0.to_owned().color(Color32::WHITE));
            ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                ui.label(format!(
                    "FPS: {}",
                    diagnostics
                        .get(&FrameTimeDiagnosticsPlugin::FPS)
                        .and_then(Diagnostic::average)
                        .map_or_else(|| "???".into(), |fps| format!("{fps:.2}")),
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
