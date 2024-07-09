use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{info_windows::InfoWindowsAct, ui::popup::Popup};

#[allow(clippy::needless_pass_by_value)]
pub fn on_manual(trigger: Trigger<InfoWindowsAct>, mut popup: EventWriter<Popup>) {
    if *trigger.event() != InfoWindowsAct::Manual {
        return;
    }
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
