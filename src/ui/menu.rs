use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, egui::Align, EguiContext};
use bevy_mouse_tracking_plugin::MousePosWorld;

use crate::{
    component_actions::undo_redo::UndoRedoAct, info_windows::InfoWindowsAct,
    load_save::LoadSaveAct, misc::Action, tilemap::settings::TileSettingsAct, ui::HoveringOverGui,
};

pub fn ui_sy(
    mut ctx: ResMut<EguiContext>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
    mut event_writer: EventWriter<Action>,
    diagnostics: Res<Diagnostics>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    let panel = egui::TopBottomPanel::top("menu").show(ctx.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            macro_rules! button {
                ($ui:ident, $ew:ident, $label:literal, $action:expr) => {
                    if $ui.button($label).clicked() {
                        info!(label = $label, "Clicked menu item");
                        $ew.send(Box::new($action))
                    }
                };
            }

            egui::menu::menu_button(
                ui,
                format!("Stencil v{}", env!("CARGO_PKG_VERSION")),
                |ui| {
                    button!(ui, event_writer, "Info", InfoWindowsAct::Info);
                    button!(ui, event_writer, "Changelog", InfoWindowsAct::Changelog);
                    button!(ui, event_writer, "Licenses", InfoWindowsAct::Licenses);
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
            });
            ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                ui.label(format!(
                    "FPS: {}",
                    diagnostics
                        .get(FrameTimeDiagnosticsPlugin::FPS)
                        .and_then(|diagnostic| diagnostic.average())
                        .map(|fps| format!("{:.2}", fps))
                        .unwrap_or_else(|| "???".into()),
                ));
                ui.separator();
                ui.label(format!(
                    "x: {} z: {}",
                    mouse_pos_world.round().x as i32,
                    -mouse_pos_world.round().y as i32
                ))
            })
        });
    });
    if panel.response.hovered() {
        hovering_over_gui.0 = true;
    }
}
