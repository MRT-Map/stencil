use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::{info_windows::InfoWindowsAct, misc::Action, ui::popup::Popup};

pub fn changelog_asy(mut actions: EventReader<Action>, mut popup: EventWriter<Popup>) {
    for event in &mut actions {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Changelog)) {
            popup.send(Popup::new(
                "changelog",
                || {
                    egui::Window::new("Changelog")
                        .collapsible(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                        .vscroll(true)
                },
                |_, ui, _, show| {
                    let mut cache = CommonMarkCache::default();
                    CommonMarkViewer::new("viewer").show(
                        ui,
                        &mut cache,
                        include_str!("../../changelog.md"),
                    );
                    if ui.button("Close").clicked() {
                        *show = false;
                    }
                },
                Mutex::new(Box::new(())),
            ));
        }
    }
}
