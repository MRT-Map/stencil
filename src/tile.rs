use crate::utils::{
    get_cursor_world_pos, get_map_coords_of_edges, get_map_width_height, get_window_width_height,
};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_mouse_tracking_plugin::{MainCamera, MousePos};

pub struct Zoom(pub f32);
impl Zoom {
    fn map_size(&self) -> f64 {
        2f64.powf((8f32 - self.0) as f64) * 32f64
    }
    fn world_size(&self) -> i32 {
        (2f64.powf((8f32 - self.0) as f64) * 32f64) as i32
    }
}

#[derive(Component, Default, PartialEq, Copy, Clone, Debug)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
    pub z: i8,
}
impl TileCoord {
    fn from_world_coords(x: f64, y: f64, z: i8) -> Self {
        Self {
            x: (x / Zoom(z as f32).world_size() as f64) as i32,
            y: (y / Zoom(z as f32).world_size() as f64) as i32,
            z,
        }
    }

    fn get_edges(&self) -> (f32, f32, f32, f32) {
        (
            self.x as f32 * Zoom(self.z as f32).world_size() as f32,
            self.y as f32 * Zoom(self.z as f32).world_size() as f32,
            (self.x + 1) as f32 * Zoom(self.z as f32).world_size() as f32,
            (self.y + 1) as f32 * Zoom(self.z as f32).world_size() as f32,
        )
    }

    fn url(&self) -> String {
        let z = 2.0f64.powi((8 - self.z) as i32);
        let x = self.x as f64;
        let y = self.y as f64;

        let group_x = ((x * z) as f64 / 32.0).floor() as i32;
        let group_y = ((y * z) as f64 / 32.0).floor() as i32;

        let num_in_group_x = x * z;
        let num_in_group_y = y * z;

        let mut zzz = "".to_string();
        let mut i = 8;
        while i > self.z {
            zzz += "z";
            i -= 1;
        }

        if !zzz.is_empty() {
            zzz += "_"
        };
        format!("http://api.allorigins.win/raw?url=https%3A//dynmap.minecartrapidtransit.net/tiles/new/flat/{}_{}/{}{}_{}.png",
            group_x, group_y, zzz, num_in_group_x, num_in_group_y)
    }
}

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
    let (camera, transform) = q_camera.single();
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

pub fn mouse_drag(
    buttons: Res<Input<MouseButton>>,
    mut mouse_origin_pos: Local<Option<MousePos>>,
    mut camera_origin_pos: Local<Option<Vec2>>,
    mouse_pos: Res<MousePos>,
    mut camera: Query<(&Camera, &mut GlobalTransform), With<MainCamera>>,
    windows: Res<Windows>,
) {
    let (camera, mut transform) = camera.single_mut();
    if buttons.pressed(MouseButton::Left) {
        if let Some(origin_pos) = *mouse_origin_pos {
            if !mouse_pos.is_changed() {
                return;
            }
            let win_wh = get_window_width_height(&windows, camera);
            let map_wh = get_map_width_height(camera, &transform);

            let dx = map_wh.x / win_wh.x * (mouse_pos.x - origin_pos.x);
            let dy = map_wh.y / win_wh.y * (mouse_pos.y - origin_pos.y);
            transform.translation.x = camera_origin_pos.unwrap().x - dx;
            transform.translation.y = camera_origin_pos.unwrap().y - dy;
        } else {
            *mouse_origin_pos = Some(*mouse_pos.into_inner());
            *camera_origin_pos = Some(transform.translation.truncate());
        }
    } else {
        *mouse_origin_pos = None;
        *camera_origin_pos = None;
    }
}

pub fn mouse_zoom(
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera: Query<
        (&Camera, &mut OrthographicProjection, &mut GlobalTransform),
        With<MainCamera>,
    >,
    mut zoom: ResMut<Zoom>,
    windows: Res<Windows>,
) {
    let (camera, mut ort_proj, mut transform) = camera.single_mut();
    for ev in scroll_evr.iter() {
        let u = match ev.unit {
            MouseScrollUnit::Line => ev.y * 0.125,
            MouseScrollUnit::Pixel => ev.y * 0.0125,
        };
        if 1.0 <= (zoom.0 + u) && (zoom.0 + u) <= 11.0 {
            zoom.0 += u;
            let orig_x = transform.translation.x;
            let orig_y = transform.translation.y;
            let orig_scale = ort_proj.scale;
            let orig_mouse_pos = get_cursor_world_pos(&windows, camera, &transform).unwrap();

            ort_proj.scale = 2f32.powf(7.0 - zoom.0);
            let dx =
                (orig_mouse_pos.x - orig_x) / (2f32.powf(orig_scale) / 2f32.powf(ort_proj.scale));
            let dy =
                (orig_mouse_pos.y - orig_y) / (2f32.powf(orig_scale) / 2f32.powf(ort_proj.scale));
            transform.translation.x = orig_mouse_pos.x - dx;
            transform.translation.y = orig_mouse_pos.y - dy;
        }
        eprintln!("{:?}", zoom.0);
    }
}
