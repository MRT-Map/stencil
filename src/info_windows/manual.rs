use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{info_windows::InfoWindowsEv, ui::popup::Popup};

#[expect(clippy::needless_pass_by_value)]
pub fn on_manual(trigger: Trigger<InfoWindowsEv>, mut popup: EventWriter<Popup>) {
    if *trigger.event() != InfoWindowsEv::Manual {
        return;
    }
    popup.write(Popup::new(
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
