use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{misc::Action, ui::popup::Popup};

pub fn tile_settings_msy(mut actions: EventReader<Action>, mut popup: EventWriter<Arc<Popup>>) {
    for event in actions.iter() {
        if event.id == "tile_settings" {
            popup.send(Popup::new(
                "tile_settings_win",
                || {
                    egui::Window::new("Tilemap Settings")
                        .resizable(true)
                        .collapsible(true)
                },
                |state, ui, ew, shown| {
                    ui.label("Coming soon...");
                    if ui.button("Close").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new(())),
            ))
        } else if event.id == "update_tile_settings" {
        }
    }
}
