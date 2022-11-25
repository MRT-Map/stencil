use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::{egui, egui::Color32};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use surf::Url;

use crate::{
    misc::{Action, DATA_DIR},
    ui::popup::Popup,
};

#[derive(Deserialize, Serialize, Clone, PartialEq, Resource)]
pub struct TileSettings {
    pub init_zoom: f32,
    pub url: String,
    pub max_tile_zoom: i8,
    pub max_zoom_range: f64,
}
impl Default for TileSettings {
    fn default() -> Self {
        Self {
            init_zoom: 7.0,
            url: "https://dynmap.minecartrapidtransit.net/tiles/new/flat".into(),
            max_tile_zoom: 8,
            max_zoom_range: 32.0,
        }
    }
}

pub static INIT_TILE_SETTINGS: Lazy<TileSettings> = Lazy::new(|| {
    let mut path = DATA_DIR.to_owned();
    path.push("tile_settings.msgpack");
    match std::fs::read(path) {
        Ok(bytes) => {
            info!("Found tile settings file");
            rmp_serde::from_slice(&bytes).unwrap()
        }
        Err(e) => {
            info!("Couldn't find or open tile settings file: {e:?}");
            TileSettings::default()
        }
    }
});

#[allow(dead_code)]
pub enum TileSettingsAct {
    Open,
    Update(TileSettings),
}

pub fn tile_settings_msy(
    mut actions: EventReader<Action>,
    mut popup: EventWriter<Arc<Popup>>,
    mut tile_settings: ResMut<TileSettings>,
) {
    for event in actions.iter() {
        if let Some(TileSettingsAct::Open) = event.downcast_ref() {
            popup.send(Popup::new(
                "tile_settings_win",
                || {
                    egui::Window::new("Tilemap Settings")
                        .resizable(true)
                        .collapsible(true)
                },
                |state, ui, ew, shown| {
                    let mut state = state.lock().unwrap();
                    let tile_settings: &mut TileSettings = state.downcast_mut().unwrap();
                    if ui.add_enabled(*tile_settings != TileSettings::default(), egui::Button::new("Reset")).clicked() {
                        *tile_settings = TileSettings::default();
                    }
                    ui.add(egui::Slider::new(&mut tile_settings.init_zoom, -10.0..=10.0)
                        .text("Initial zoom"));
                    ui.label("How zoomed in the map is when the app is first opened. Larger values mean more zoomed in");
                    ui.separator();
                    ui.label("Unless you're using a different tilemap, you shouldn't need to change anything below here");
                    ui.add(egui::Slider::new(&mut tile_settings.max_tile_zoom, -5..=15)
                        .text("Maximum tile zoom"));
                    ui.label("...I don't know how to explain this");
                    ui.add(egui::Slider::new(&mut tile_settings.max_zoom_range, 1.0..=256.0)
                        .text("Maximum tile zoom range"));
                    ui.label("In tiles of the highest zoom level, the distance across its width / height that each tile represents");
                    ui.add(egui::TextEdit::singleline(&mut tile_settings.url).hint_text("Base URL"));
                    if let Err(e) = Url::try_from(&*tile_settings.url) {
                        ui.colored_label(Color32::RED, format!("Invalid URL: {e:?}"));
                    }
                    ui.label("The base URL of the tile source");
                    ui.separator();
                    if ui.button("Save").clicked() {
                        ew.send(Box::new(TileSettingsAct::Update(tile_settings.to_owned())));
                        *shown = false;
                    }
                    if ui.button("Cancel").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new(tile_settings.to_owned())),
            ))
        } else if let Some(TileSettingsAct::Update(new_settings)) = event.downcast_ref() {
            *tile_settings = new_settings.to_owned();
            let mut path = DATA_DIR.to_owned();
            path.push("tile_settings.msgpack");
            std::fs::write(path, rmp_serde::to_vec(new_settings).unwrap()).unwrap();
        }
    }
}
