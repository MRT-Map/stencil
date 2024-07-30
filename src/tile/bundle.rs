use bevy::{prelude::*, sprite::Anchor};

use crate::{
    tile::{tile_coord::TileCoord, zoom::Zoom},
    ui::tilemap::settings::Basemap,
};

#[derive(Component)]
pub struct Tile;

#[expect(clippy::partial_pub_fields)]
#[derive(Bundle)]
pub struct TileBundle {
    _t: Tile,
    pub coord: TileCoord,

    pub sprite: SpriteBundle,
}

impl TileBundle {
    pub fn from_tile_coord(coord: TileCoord, server: &Res<AssetServer>, basemap: &Basemap) -> Self {
        let custom_size = Vec2::splat(Zoom(f32::from(coord.z)).map_size(basemap) as f32);
        trace!(coord = coord.to_string(), "Loading tile");
        Self {
            _t: Tile,
            coord,
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(custom_size),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                texture: server.load(coord.path(basemap)),
                transform: Transform::from_translation(Vec3::new(
                    (coord.x as f32)
                        .mul_add(Zoom(f32::from(coord.z)).map_size(basemap) as f32, -0.5f32),
                    (coord.y as f32).mul_add(
                        Zoom(f32::from(coord.z)).map_size(basemap) as f32,
                        basemap.max_zoom_range as f32,
                    ) + 0.5f32,
                    0.0,
                )),
                ..default()
            },
        }
    }
}
