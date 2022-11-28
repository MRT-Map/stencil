use bevy::{prelude::*, sprite::Anchor};

use crate::tilemap::{settings::TileSettings, tile_coord::TileCoord, zoom::Zoom};

#[derive(Component)]
pub struct Tile;

#[derive(Bundle)]
pub struct TileBundle {
    _t: Tile,
    pub coord: TileCoord,

    #[bundle]
    pub sprite: SpriteBundle,
}

impl TileBundle {
    pub fn from_tile_coord(
        coord: TileCoord,
        server: &Res<AssetServer>,
        tile_settings: &TileSettings,
    ) -> Self {
        let custom_size = Vec2::splat(Zoom(f32::from(coord.z)).map_size(tile_settings) as f32);
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
                texture: server.load(&*coord.path(tile_settings)) as Handle<Image>,
                transform: Transform::from_translation(Vec3::new(
                    (coord.x as f32).mul_add(
                        Zoom(f32::from(coord.z)).map_size(tile_settings) as f32,
                        -0.5f32,
                    ),
                    (coord.y as f32).mul_add(
                        Zoom(f32::from(coord.z)).map_size(tile_settings) as f32,
                        tile_settings.max_zoom_range as f32,
                    ) + 0.5f32,
                    0.0,
                )),
                ..default()
            },
        }
    }
}
