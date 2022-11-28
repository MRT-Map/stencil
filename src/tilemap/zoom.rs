use bevy::prelude::*;

use crate::tilemap::settings::TileSettings;

#[derive(Copy, Clone, Debug, Resource)]
pub struct Zoom(pub f32);

impl Zoom {
    pub fn map_size(self, tile_settings: &TileSettings) -> f64 {
        f64::from(f32::from(tile_settings.max_tile_zoom) - self.0).exp2()
            * tile_settings.max_zoom_range
    }
    pub fn world_size(self, tile_settings: &TileSettings) -> i32 {
        (f64::from(f32::from(tile_settings.max_tile_zoom) - self.0).exp2()
            * tile_settings.max_zoom_range) as i32
    }
}
