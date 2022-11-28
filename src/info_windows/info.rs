use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{info_windows::InfoWindowsAct, misc::Action, ui::popup::Popup};

pub fn info_asy(mut actions: EventReader<Action>, mut popup: EventWriter<Arc<Popup>>) {
    for event in actions.iter() {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Info)) {
            popup.send(Popup::new(
                "info_popup",
                || {
                    egui::Window::new(format!("Stencil v{}", env!("CARGO_PKG_VERSION")))
                        .collapsible(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                },
                |_, ui, _, shown| {
                    ui.label("Made by __7d for the MRT Mapping Services");
                    ui.hyperlink_to("GitHub", "https://github.com/MRT-Map/stencil2");
                    if ui.button("Close").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new(())),
            ));
        }
    }
}
