use bevy::prelude::*;
use itertools::Either;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::misc::data_path;

macro_rules! field {
    ($f:ident, $f2:ident, $i:ident, $t:ty) => {
        #[allow(clippy::float_cmp)]
        fn $f(v: &$t) -> bool {
            *v == TileSettings::default().$i
        }
        fn $f2() -> $t {
            TileSettings::default().$i
        }
    };
}
field!(init_zoom_is_default, default_init_zoom, init_zoom, f32);
field!(url_is_default, default_url, url, String);
field!(
    max_tile_zoom_is_default,
    default_max_tile_zoom,
    max_tile_zoom,
    i8
);
field!(
    max_zoom_range_is_default,
    default_max_zoom_range,
    max_zoom_range,
    f64
);
field!(
    max_get_requests_is_default,
    default_max_get_requests,
    max_get_requests,
    usize
);
field!(
    clear_cache_on_startup_is_default,
    default_clear_cache_on_startup,
    clear_cache_on_startup,
    bool
);

#[derive(Deserialize, Serialize, Clone, PartialEq, Resource)]
pub struct TileSettings {
    #[serde(
        default = "default_init_zoom",
        skip_serializing_if = "init_zoom_is_default"
    )]
    pub init_zoom: f32,
    #[serde(default = "default_url", skip_serializing_if = "url_is_default")]
    pub url: String,
    #[serde(
        default = "default_max_tile_zoom",
        skip_serializing_if = "max_tile_zoom_is_default"
    )]
    pub max_tile_zoom: i8,
    #[serde(
        default = "default_max_zoom_range",
        skip_serializing_if = "max_zoom_range_is_default"
    )]
    pub max_zoom_range: f64,
    #[serde(
        default = "default_max_get_requests",
        skip_serializing_if = "max_get_requests_is_default"
    )]
    pub max_get_requests: usize,
    #[serde(
        default = "default_clear_cache_on_startup",
        skip_serializing_if = "clear_cache_on_startup_is_default"
    )]
    pub clear_cache_on_startup: bool,
}

impl Default for TileSettings {
    fn default() -> Self {
        Self {
            init_zoom: 7.0,
            url: "https://dynmap.minecartrapidtransit.net/tiles/new/flat".into(),
            max_tile_zoom: 8,
            max_zoom_range: 32.0,
            max_get_requests: 50,
            clear_cache_on_startup: false,
        }
    }
}

impl TileSettings {
    pub fn load() -> Result<Self, toml::de::Error> {
        match std::fs::read_to_string(data_path("tile_settings.toml")) {
            Ok(str) => {
                info!("Found tile settings file");
                toml::from_str(&str)
            }
            Err(e) => {
                info!("Couldn't find or open tile settings file: {e:?}");
                let s = Self::default();
                let _ = s.save();
                Ok(s)
            }
        }
    }
    pub fn save(&self) -> Result<(), Either<std::io::Error, toml::ser::Error>> {
        info!("Saving tile settings file");
        let prefix_text = "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#tile_settingstoml";
        let serialized = toml::to_string_pretty(self).map_err(Either::Right)?;

        std::fs::write(
            data_path("tile_settings.toml"),
            format!("{prefix_text}\n\n{serialized}"),
        )
        .map_err(Either::Left)
    }
}

pub static INIT_TILE_SETTINGS: Lazy<TileSettings> = Lazy::new(|| TileSettings::load().unwrap());
