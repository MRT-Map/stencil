use std::{path::PathBuf, sync::LazyLock};

use lazy_regex::{Regex, lazy_regex};
use serde::{Deserialize, Serialize};

use crate::{
    dirs_paths::cache_dir,
    map::{settings::MapSettings, tile_coord::TileCoord},
    settings::misc_settings::MiscSettings,
};

pub static URL_REPLACER: LazyLock<Regex> = lazy_regex!("[<>:/\\|?*\"]");

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BasemapURLType {
    DynMap,
    Regular,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Basemap {
    pub url: String,
    pub extension: String,
    pub max_tile_zoom: i8,
    pub max_zoom_world_size: f32,
    pub url_type: BasemapURLType,
}

impl Default for Basemap {
    fn default() -> Self {
        Self {
            url: "https://dynmap.minecartrapidtransit.net/main/tiles/new/flat".into(),
            extension: "png".into(),
            max_tile_zoom: 8,
            max_zoom_world_size: 32.0,
            url_type: BasemapURLType::DynMap,
        }
    }
}

impl Basemap {
    #[must_use]
    pub fn url(&self, tile_coord: TileCoord) -> String {
        match self.url_type {
            BasemapURLType::DynMap => {
                let z = 2.0f32.powi(i32::from(self.max_tile_zoom - tile_coord.z));
                let xy = egui::vec2(tile_coord.x as f32, tile_coord.y as f32);

                let group = (xy * z / self.max_zoom_world_size).floor();

                let num_in_group = xy * z;

                let mut zzz = String::new();
                let mut i = self.max_tile_zoom;
                while i > tile_coord.z {
                    zzz += "z";
                    i -= 1;
                }

                if !zzz.is_empty() {
                    zzz += "_";
                }

                format!(
                    "{}/{}_{}/{zzz}{}_{}.{}",
                    self.url,
                    group.x as i32,
                    group.y as i32,
                    num_in_group.x as i32,
                    num_in_group.y as i32,
                    self.extension
                )
            }
            BasemapURLType::Regular => {
                format!(
                    "{}/{}/{}/{}.{}",
                    self.url, tile_coord.z, tile_coord.x, tile_coord.y, self.extension
                )
            }
        }
    }
    #[must_use]
    pub fn cache_path(&self) -> PathBuf {
        cache_dir("tile-cache").join(URL_REPLACER.replace_all(&self.url, "").as_ref())
    }
}

impl Basemap {
    pub fn tile_zoom(&self, zoom: f32) -> i8 {
        (zoom.round() as i8).clamp(0, self.max_tile_zoom)
    }
    pub fn tile_world_size(&self, zoom: i8) -> f32 {
        self.max_zoom_world_size * 2.0f32.powi(i32::from(self.max_tile_zoom - zoom))
    }
    pub fn tile_screen_size(&self, map_settings: &MapSettings, zoom: i8) -> u32 {
        (self.tile_world_size(zoom)
            / map_settings.world_screen_ratio_at_zoom(self.max_tile_zoom, zoom as f32))
            as u32
    }
}
