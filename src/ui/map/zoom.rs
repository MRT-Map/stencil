use bevy::prelude::*;

use crate::ui::map::settings::Basemap;

#[derive(Copy, Clone, Debug, Resource)]
pub struct Zoom(pub f32);

impl Zoom {
    #[must_use]
    pub fn tile_size(self, basemap: &Basemap) -> f32 {
        (f32::from(basemap.max_tile_zoom) - self.0).exp2() * basemap.max_zoom_range
    }
}
