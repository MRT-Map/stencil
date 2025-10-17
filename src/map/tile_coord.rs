use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    sync::LazyLock,
};

use lazy_regex::{Regex, lazy_regex};

use crate::{dirs_paths::cache_dir, map::basemap::Basemap};

#[derive(Default, PartialEq, Eq, Copy, Clone, Debug, Hash)]
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
    pub fn cache_path(&self, basemap: &Basemap) -> PathBuf {
        let path = basemap
            .cache_path()
            .join(self.z.to_string())
            .join(self.x.to_string());
        let _ = std::fs::create_dir_all(&path);
        path.join(format!("{}.{}", self.y, basemap.extension))
    }
}
