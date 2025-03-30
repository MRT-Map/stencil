use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use bevy::prelude::*;
use lazy_regex::{lazy_regex, Regex};
use once_cell::sync::Lazy;

use crate::{dirs_paths::cache_dir, tile::zoom::Zoom, ui::tilemap::settings::Basemap};

#[expect(clippy::non_std_lazy_statics)]
pub static URL_REPLACER: Lazy<Regex> = lazy_regex!("[<>:/\\|?*\"]");

#[derive(Component, Default, PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
    pub z: i8,
}

impl Display for TileCoord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.z, self.y, self.x)
    }
}

impl TileCoord {
    #[must_use]
    pub fn from_world_coords(x: f64, y: f64, z: i8, basemap: &Basemap) -> Self {
        Self {
            x: (x / Zoom(f32::from(z)).world_size(basemap)) as i32,
            y: (y / Zoom(f32::from(z)).world_size(basemap)) as i32,
            z,
        }
    }

    #[must_use]
    pub fn get_edges(&self, basemap: &Basemap) -> (f32, f32, f32, f32) {
        (
            self.x as f32 * Zoom(f32::from(self.z)).world_size(basemap) as f32,
            self.y as f32 * Zoom(f32::from(self.z)).world_size(basemap) as f32,
            (self.x + 1) as f32 * Zoom(f32::from(self.z)).world_size(basemap) as f32,
            (self.y + 1) as f32 * Zoom(f32::from(self.z)).world_size(basemap) as f32,
        )
    }

    #[must_use]
    pub fn url(&self, basemap: &Basemap) -> String {
        let z = 2.0f64.powi(i32::from(basemap.max_tile_zoom - self.z));
        let xy = IVec2::new(self.x, self.y).as_dvec2();

        let group = (xy * z / basemap.max_zoom_range).floor().as_ivec2();

        let num_in_group = xy * z;

        let mut zzz = String::new();
        let mut i = basemap.max_tile_zoom;
        while i > self.z {
            zzz += "z";
            i -= 1;
        }

        if !zzz.is_empty() {
            zzz += "_";
        }
        format!(
            "{}/{}_{}/{zzz}{}_{}.{}",
            basemap.url, group.x, group.y, num_in_group.x, num_in_group.y, basemap.extension
        )
    }

    pub fn path(&self, basemap: &Basemap) -> PathBuf {
        let path = cache_dir("tile-cache")
            .join(URL_REPLACER.replace_all(&basemap.url, "").as_ref())
            .join(self.z.to_string())
            .join(self.x.to_string());
        let _ = std::fs::create_dir_all(&path);
        path.join(format!("{}.{}", self.y, basemap.extension))
    }
}
