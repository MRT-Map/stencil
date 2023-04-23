use bevy::prelude::*;
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

pub static INIT_TILE_SETTINGS: Lazy<TileSettings> =
    Lazy::new(|| match std::fs::read(data_file("tile_settings.msgpack")) {
        Ok(bytes) => {
            info!("Found tile settings file");
            rmp_serde::from_slice(&bytes).unwrap()
        }
        Err(e) => {
            info!("Couldn't find or open tile settings file: {e:?}");
            TileSettings::default()
        }
    });
