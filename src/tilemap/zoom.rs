use bevy::prelude::*;

use crate::tilemap::settings::TileSettings;

#[derive(Copy, Clone, Debug, Resource)]
pub struct Zoom(pub f32);

impl Zoom {
    pub fn map_size(&self, tile_settings: &TileSettings) -> f64 {
        2f64.powf((tile_settings.max_tile_zoom as f32 - self.0) as f64)
            * tile_settings.max_zoom_range
    }
    pub fn world_size(&self, tile_settings: &TileSettings) -> i32 {
        (2f64.powf((tile_settings.max_tile_zoom as f32 - self.0) as f64)
            * tile_settings.max_zoom_range) as i32
    }
}
