use bevy::prelude::*;

use crate::ui::tilemap::settings::Basemap;

#[derive(Copy, Clone, Debug, Resource)]
pub struct Zoom(pub f32);

impl Zoom {
    #[must_use]
    pub fn map_size(self, basemap: &Basemap) -> f64 {
        f64::from(f32::from(basemap.max_tile_zoom) - self.0).exp2() * basemap.max_zoom_range
    }
    #[must_use]
    pub fn world_size(self, basemap: &Basemap) -> f64 {
        f64::from(f32::from(basemap.max_tile_zoom) - self.0).exp2() * basemap.max_zoom_range
    }
}
