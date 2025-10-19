use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    URL_REPLACER,
    file::{cache_dir, safe_delete},
    map::{settings::MapSettings, tile_coord::TileCoord},
    settings::misc_settings::MiscSettings,
    ui::notif::NotifState,
};

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
    pub offset: (f32, f32),
}

impl Default for Basemap {
    fn default() -> Self {
        Self {
            url: "https://dynmap.minecartrapidtransit.net/main/tiles/new/flat".into(),
            extension: "png".into(),
            max_tile_zoom: 8,
            max_zoom_world_size: 32.0,
            url_type: BasemapURLType::DynMap,
            offset: (0.5, 32.5),
        }
    }
}

impl Basemap {
    #[must_use]
    pub fn url(&self, tile_coord: TileCoord) -> String {
        match self.url_type {
            BasemapURLType::DynMap => {
                let z = 2.0f32.powi(i32::from(self.max_tile_zoom - tile_coord.z));
                let xy = egui::vec2(tile_coord.x as f32, -tile_coord.y as f32);

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
    pub fn clear_cache_path(&self, misc_settings: &MiscSettings, notifs: &mut NotifState) {
        let _ = safe_delete(self.cache_path(), misc_settings, notifs);
    }
}

impl Basemap {
    pub fn tile_zoom(&self, zoom: f32) -> i8 {
        (zoom.round() as i8).clamp(0, self.max_tile_zoom)
    }
    pub fn tile_world_size(&self, zoom: i8) -> f32 {
        self.max_zoom_world_size * 2.0f32.powi(i32::from(self.max_tile_zoom - zoom))
    }
    pub fn tile_screen_size(&self, map_settings: &MapSettings, zoom: f32) -> f32 {
        self.tile_world_size(self.tile_zoom(zoom))
            / map_settings.world_screen_ratio_at_zoom(self.max_tile_zoom, zoom)
    }
}
