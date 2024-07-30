use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::{info_windows::InfoWindowsEv, ui::popup::Popup};

#[expect(clippy::needless_pass_by_value)]
pub fn on_changelog(trigger: Trigger<InfoWindowsEv>, mut popup: EventWriter<Popup>) {
    if *trigger.event() != InfoWindowsEv::Changelog {
        return;
    }
    popup.send(Popup::new(
        "changelog",
        || {
            egui::Window::new("Changelog")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        },
        |_, ui, _, shown| {
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() * 0.75)
                .show(ui, |ui| {
                    let mut cache = CommonMarkCache::default();
                    CommonMarkViewer::new("viewer").show(
                        ui,
                        &mut cache,
                        include_str!("../../changelog.md"),
                    );
                });
            ui.separator();
            if ui.button("Close").clicked() {
                *shown = false;
            }
        },
        Mutex::new(Box::new(())),
    ));
}
