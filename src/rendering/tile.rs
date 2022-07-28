use crate::rendering::utils::{
    get_cursor_world_pos, get_map_coords_of_edges, get_map_width_height, get_window_width_height,
};
use crate::types::TileCoord;
use crate::Zoom;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_mouse_tracking_plugin::{MainCamera, MousePos};

#[derive(Component)]
pub struct Tile;
impl Drop for Tile {
    // appease clippy
    fn drop(&mut self) {}
}

#[derive(Bundle)]
pub struct TileBundle {
    _t: Tile,
    pub coord: TileCoord,

    #[bundle]
    pub sprite: SpriteBundle,
}
impl TileBundle {
    pub fn from_tile_coord(coord: TileCoord, server: &Res<AssetServer>) -> Self {
        let custom_size = Vec2::new(
            Zoom(coord.z as f32).map_size() as f32,
            Zoom(coord.z as f32).map_size() as f32,
        );
        Self {
            _t: Tile,
            coord,
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(custom_size),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                texture: server.load(&*coord.url()) as Handle<Image>,
                transform: Transform::from_translation(Vec3::new(
                    coord.x as f32 * Zoom(coord.z as f32).map_size() as f32,
                    coord.y as f32 * Zoom(coord.z as f32).map_size() as f32 + 32f32,
                    coord.z as f32,
                )),
                ..default()
            },
        }
    }
}

pub fn get_shown_tiles(
    q_camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    zoom: i8,
) -> Vec<TileCoord> {
    let (camera, transform): (&Camera, &GlobalTransform) = q_camera.single();
    let (c_left, c_top, c_right, c_bottom) = get_map_coords_of_edges(camera, transform);
    let TileCoord {
        x: t_left,
        y: t_top,
        ..
    } = TileCoord::from_world_coords(c_left as f64, c_top as f64, zoom);
    let TileCoord {
        x: t_right,
        y: t_bottom,
        ..
    } = TileCoord::from_world_coords(c_right as f64, c_bottom as f64, zoom);

    (t_left - 1..=t_right + 1)
        .flat_map(|ref x| {
            (t_top - 1..=t_bottom + 1)
                .map(|y| TileCoord { x: *x, y, z: zoom })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn show_tiles(
    mut commands: Commands,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query: Query<(&mut Visibility, &TileCoord), With<Tile>>,
    zoom: Res<Zoom>,
    server: Res<AssetServer>,
) {
    //if !zoom.is_changed() {return}
    let mut shown_tiles = get_shown_tiles(&q_camera, zoom.0.round() as i8);

    let (camera, transform): (&Camera, &GlobalTransform) = q_camera.single();
    let (ml, mt, mr, mb) = get_map_coords_of_edges(camera, transform);
    for (mut visibility, tile_coord) in query.iter_mut() {
        if (zoom.0 <= 8f32 && tile_coord.z > zoom.0.round() as i8)
            || (zoom.0 > 8f32 && tile_coord.z != 8)
            || (zoom.0 > 8f32 && {
                let (tl, tt, tr, tb) = tile_coord.get_edges();
                tr < ml || tl > mr || tb < mt || tt > mb
            })
            || (tile_coord.z <= 7 && zoom.0 <= 8f32 && !shown_tiles.contains(tile_coord))
        {
            visibility.is_visible = false;
        } else {
            shown_tiles.retain(|t| t != tile_coord);
            visibility.is_visible = true;
        }
    }
    for tile_coord in shown_tiles {
        if tile_coord.z <= 8 {
            commands.spawn_bundle(TileBundle::from_tile_coord(tile_coord, &server));
        }
    }
}
