use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, egui::Align, EguiContext};

use crate::editor::{menu_actions::MenuAction, ui::HoveringOverGui};

pub fn ui_sy(
    mut ctx: ResMut<EguiContext>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
    mut event_writer: EventWriter<MenuAction>,
    diagnostics: Res<Diagnostics>,
) {
    let panel = egui::TopBottomPanel::top("menu").show(ctx.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            macro_rules! button {
                ($ui:ident, $ew:ident, $label:literal, $id:literal) => {
                    if $ui.button($label).clicked() {
                        $ew.send(MenuAction($id))
                    }
                };
            }

            egui::menu::menu_button(
                ui,
                format!("Stencil v{}", env!("CARGO_PKG_VERSION")),
                |ui| {
                    button!(ui, event_writer, "Info", "info");
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
            ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                ui.label(format!(
                    "FPS: {}",
                    if let Some(diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                        if let Some(fps) = diagnostic.average() {
                            format!("{:.2}", fps)
                        } else {
                            "???".to_string()
                        }
                    } else {
                        "???".to_string()
                    }
                ));
            })
        });
    });
    if panel.response.hovered() {
        hovering_over_gui.0 = true;
    }
}
