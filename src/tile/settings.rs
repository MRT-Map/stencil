use bevy::prelude::*;
use itertools::Either;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::misc::data_file;

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

impl TileSettings {
    pub fn load() -> Result<Self, toml::de::Error> {
        match std::fs::read_to_string(data_file("tile_settings.toml")) {
            Ok(str) => {
                info!("Found tile settings file");
                toml::from_str(&str)
            }
            Err(e) => {
                info!("Couldn't find or open tile settings file: {e:?}");
                let s = TileSettings::default();
                let _ = s.save();
                Ok(s)
            }
        }
    }
    pub fn save(&self) -> Result<(), Either<std::io::Error, toml::ser::Error>> {
        info!("Saving tile settings file");
        let prefix_text = "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#tile_settingstoml";
        let serialized = toml::to_string_pretty(self).map_err(|a| Either::Right(a))?;

        std::fs::write(
            data_file("tile_settings.toml"),
            format!("{prefix_text}\n\n{serialized}"),
        )
        .map_err(|a| Either::Left(a))
    }
}

pub static INIT_TILE_SETTINGS: Lazy<TileSettings> = Lazy::new(|| TileSettings::load().unwrap());
