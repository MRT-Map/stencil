use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::{action::Action, info_windows::InfoWindowsAct, ui::popup::Popup};

pub fn changelog_asy(mut actions: EventReader<Action>, mut popup: EventWriter<Popup>) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Changelog)) {
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
    }
}
