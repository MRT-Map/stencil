use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{action::Action, info_windows::InfoWindowsAct, ui::popup::Popup};

pub fn manual_asy(mut actions: EventReader<Action>, mut popup: EventWriter<Popup>) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Manual)) {
            popup.send(Popup::new(
                "manual",
                || {
                    egui::Window::new("Manual")
                        .collapsible(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                        .resizable(false)
                },
                |_, ui, _, shown| {
                    ui.label("Our online manual is available here:");
                    ui.hyperlink("https://github.com/MRT-Map/stencil2/wiki");
                    if ui.button("Close").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new(())),
            ));
        }
    }
}
