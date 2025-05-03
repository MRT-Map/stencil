use bevy::{
    asset::AssetServer,
    math::{Vec2, Vec3},
    prelude::*,
    sprite::Anchor,
};
use tracing::trace;

use crate::{
    tile::tile_coord::TileCoord,
    ui::map::{settings::Basemap, zoom::Zoom},
};

pub mod tile_coord;

#[derive(Component)]
pub struct Tile;

pub fn make_tile(coord: TileCoord, server: &Res<AssetServer>, basemap: &Basemap) -> impl Bundle {
    let custom_size = Vec2::splat(Zoom(f32::from(coord.z)).tile_size(basemap));
    trace!(coord = coord.to_string(), "Loading tile");
    (
        Tile,
        coord,
        Sprite {
            custom_size: Some(custom_size),
            anchor: Anchor::TopLeft,
            image: server.load(coord.path(basemap)),
            ..default()
        },
        Transform::from_translation(Vec3::new(
            (coord.x as f32).mul_add(Zoom(f32::from(coord.z)).tile_size(basemap), -0.5f32),
            (coord.y as f32).mul_add(
                Zoom(f32::from(coord.z)).tile_size(basemap),
                basemap.max_zoom_range,
            ) + 0.5f32,
            0.0,
        )),
    )
}
