use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{misc::Action, ui::popup::Popup};

#[allow(dead_code)]
pub enum TileSettingsAct {
    Open,
    Update,
}

pub fn tile_settings_msy(mut actions: EventReader<Action>, mut popup: EventWriter<Arc<Popup>>) {
    for event in actions.iter() {
        if let Some(TileSettingsAct::Open) = event.downcast_ref() {
            popup.send(Popup::new(
                "tile_settings_win",
                || {
                    egui::Window::new("Tilemap Settings")
                        .resizable(true)
                        .collapsible(true)
                },
                |_, ui, _, shown| {
                    ui.label("Coming soon...");
                    if ui.button("Close").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new(())),
            ))
        } else if let Some(TileSettingsAct::Update) = event.downcast_ref() {
        }
    }
}
