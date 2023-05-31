use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::{egui, egui::Color32};
use surf::Url;

use crate::{
    misc::{data_path, Action},
    tile::settings::TileSettings,
    ui::popup::Popup,
};

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
        if matches!(event.downcast_ref(), Some(TileSettingsAct::Open)) {
            popup.send(Popup::new(
                "tile_settings_win",
                || {
                    egui::Window::new("Tilemap Settings")
                        .resizable(true)
                        .collapsible(true)
                },
                |state, ui, ew, shown| {
                    let mut state = state.lock().unwrap();
                    let mut invalid = false;
                    let tile_settings: &mut TileSettings = state.downcast_mut().unwrap();
                    if ui.add_enabled(*tile_settings != TileSettings::default(), egui::Button::new("Reset")).clicked() {
                        *tile_settings = TileSettings::default();
                    }
                    ui.colored_label(Color32::YELLOW, format!("Tile settings can also be edited at: {}", data_path("tile_settings.toml").to_string_lossy()));
                    ui.separator();
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
                        invalid = true;
                    }
                    ui.label("The base URL of the tile source");
                    ui.separator();
                    if ui.add_enabled(!invalid, egui::Button::new("Save")).clicked() {
                        ew.send(Box::new(TileSettingsAct::Update(tile_settings.to_owned())));
                        *shown = false;
                    }
                    if ui.button("Cancel").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new(tile_settings.to_owned())),
            ));
        } else if let Some(TileSettingsAct::Update(new_settings)) = event.downcast_ref() {
            *tile_settings = new_settings.to_owned();
            new_settings.save().unwrap()
        }
    }
}
