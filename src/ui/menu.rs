use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, egui::Align, EguiContext};
use bevy_mouse_tracking_plugin::MousePosWorld;

use crate::{misc::Action, ui::HoveringOverGui};

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
                ($ui:ident, $ew:ident, $label:literal, $id:literal, $payload:expr) => {
                    if $ui.button($label).clicked() {
                        $ew.send(Action {
                            id: $id.into(),
                            payload: Box::new($payload),
                        })
                    }
                };
                ($ui:ident, $ew:ident, $label:literal, $id:literal) => {
                    button!($ui, $ew, $label, $id, ())
                };
            }

            egui::menu::menu_button(
                ui,
                format!("Stencil v{}", env!("CARGO_PKG_VERSION")),
                |ui| {
                    button!(ui, event_writer, "Info", "info");
                    button!(ui, event_writer, "Changelog", "changelog");
                    button!(ui, event_writer, "Quit", "quit");
                },
            );
            egui::menu::menu_button(ui, "File", |ui| {
                button!(ui, event_writer, "Load namespace", "load_ns");
                button!(ui, event_writer, "Save namespaces", "save_ns");
            });
            egui::menu::menu_button(ui, "Edit", |ui| {
                ui.label("Coming soon");
            });
            egui::menu::menu_button(ui, "Settings", |ui| {
                button!(ui, event_writer, "Tilemap", "tile_settings");
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
