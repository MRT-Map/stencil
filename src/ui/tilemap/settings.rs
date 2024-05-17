use bevy::prelude::*;
use itertools::Either;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::log::AddToErrorLog, misc::data_path};

macro_rules! field {
    ($s:ty, $f:ident, $f2:ident, $i:ident, $t:ty) => {
        #[allow(clippy::float_cmp)]
        fn $f(v: &$t) -> bool {
            *v == <$s>::default().$i
        }
        fn $f2() -> $t {
            <$s>::default().$i
        }
    };
}
field!(
    TileSettings,
    init_zoom_is_default,
    default_init_zoom,
    init_zoom,
    f32
);
field!(
    TileSettings,
    max_get_requests_is_default,
    default_max_get_requests,
    max_get_requests,
    usize
);
field!(
    TileSettings,
    clear_cache_on_startup_is_default,
    default_clear_cache_on_startup,
    clear_cache_on_startup,
    bool
);
field!(
    TileSettings,
    basemaps_is_default,
    default_basemaps,
    basemaps,
    Vec<Basemap>
);

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Basemap {
    pub url: String,
    pub max_tile_zoom: i8,
    pub max_zoom_range: f64,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Resource)]
pub struct TileSettings {
    #[serde(
        default = "default_init_zoom",
        skip_serializing_if = "init_zoom_is_default"
    )]
    pub init_zoom: f32,
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
    #[serde(
        default = "default_basemaps",
        skip_serializing_if = "basemaps_is_default"
    )]
    pub basemaps: Vec<Basemap>,
}

impl Default for TileSettings {
    fn default() -> Self {
        Self {
            init_zoom: 7.0,
            max_get_requests: 50,
            clear_cache_on_startup: false,
            basemaps: vec![Basemap::default()],
        }
    }
}

impl Default for Basemap {
    fn default() -> Self {
        Self {
            url: "https://dynmap.minecartrapidtransit.net/main/tiles/new/flat".into(),
            max_tile_zoom: 8,
            max_zoom_range: 32.0,
        }
    }
}

impl TileSettings {
    pub fn load() -> color_eyre::Result<Self> {
        match std::fs::read_to_string(data_path("tile_settings.toml")) {
            Ok(str) => {
                info!("Found tile settings file");
                Ok(toml::from_str(&str)?)
            }
            Err(e) => {
                info!("Couldn't find or open tile settings file: {e:?}");
                let s = Self::default();
                let _ = s.save();
                Ok(s)
            }
        }
    }
    pub fn save(&self) -> color_eyre::Result<()> {
        info!("Saving tile settings file");
        let prefix_text = "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#tile_settingstoml";
        let serialized = toml::to_string_pretty(self)?;

        Ok(std::fs::write(
            data_path("tile_settings.toml"),
            format!("{prefix_text}\n\n{serialized}"),
        )?)
    }
}

pub static INIT_TILE_SETTINGS: Lazy<TileSettings> =
    Lazy::new(|| TileSettings::load().unwrap_or_default_and_log());
