use std::collections::HashMap;

use async_lock::Semaphore;
use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_mouse_tracking::MainCamera;
use futures_lite::future;
use image::{ImageFormat, Rgba, RgbaImage};
use once_cell::sync::Lazy;

use crate::{
    tile::{
        bundle::{Tile, TileBundle},
        tile_coord::TileCoord,
        utils::get_map_coords_of_edges,
        zoom::Zoom,
    },
    ui::tilemap::settings::{TileSettings, INIT_TILE_SETTINGS},
};

static SEMAPHORE: Lazy<Semaphore> =
    Lazy::new(|| Semaphore::new(INIT_TILE_SETTINGS.max_get_requests));

#[must_use]
pub fn get_shown_tiles(
    q_camera: &Query<(&Camera, Ref<Transform>), impl ReadOnlyWorldQuery>,
    zoom: i8,
    tile_settings: &TileSettings,
) -> Vec<TileCoord> {
    let (camera, transform) = q_camera.single();
    let (c_left, c_top, c_right, c_bottom) = get_map_coords_of_edges(camera, &transform);
    let TileCoord {
        x: t_left,
        y: t_top,
        ..
    } = TileCoord::from_world_coords(
        f64::from(c_left),
        f64::from(c_top),
        zoom.min(tile_settings.max_tile_zoom),
        tile_settings,
    );
    let TileCoord {
        x: t_right,
        y: t_bottom,
        ..
    } = TileCoord::from_world_coords(
        f64::from(c_right),
        f64::from(c_bottom),
        zoom.min(tile_settings.max_tile_zoom),
        tile_settings,
    );

    (t_left - 1..=t_right + 1)
        .flat_map(|ref x| {
            (t_top - 1..=t_bottom + 1)
                .map(|y| TileCoord {
                    x: *x,
                    y,
                    z: zoom.min(tile_settings.max_tile_zoom),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[tracing::instrument(skip_all)]
pub fn show_tiles_sy(
    mut commands: Commands,
    q_camera: Query<(&Camera, Ref<Transform>), With<MainCamera>>,
    mut query: Query<(Entity, &TileCoord), With<Tile>>,
    zoom: Res<Zoom>,
    server: Res<AssetServer>,
    tile_settings: Res<TileSettings>,
    mut pending_tiles: Local<HashMap<TileCoord, Task<surf::Result<()>>>>,
) {
    if q_camera.is_empty() {
        return;
    }

    let (camera, transform) = q_camera.single();
    let mut shown_tiles = get_shown_tiles(&q_camera, zoom.0.round() as i8, &tile_settings);
    let thread_pool = AsyncComputeTaskPool::get();
    if !transform.is_changed() {
        let (ml, mt, mr, mb) = get_map_coords_of_edges(camera, &transform);
        for (entity, tile_coord) in query.iter_mut() {
            if (zoom.0 <= f32::from(tile_settings.max_tile_zoom)
                && tile_coord.z > zoom.0.round() as i8)
                || (zoom.0 > f32::from(tile_settings.max_tile_zoom)
                    && tile_coord.z != tile_settings.max_tile_zoom)
                || (zoom.0 > f32::from(tile_settings.max_tile_zoom) && {
                    let (tl, tt, tr, tb) = tile_coord.get_edges(&tile_settings);
                    tr < ml || tl > mr || tb < mt || tt > mb
                })
                || (tile_coord.z <= (tile_settings.max_tile_zoom - 1)
                    && zoom.0 <= f32::from(tile_settings.max_tile_zoom)
                    && !shown_tiles.contains(tile_coord))
            {
                trace!("Hiding {tile_coord}");
                commands.entity(entity).despawn_recursive();
            } else {
                shown_tiles.retain(|t| t != tile_coord);
                trace!("Showing {tile_coord}");
            }
        }
        for tile_coord in &shown_tiles {
            if tile_coord.z <= tile_settings.max_tile_zoom {
                trace!("Loading tile {tile_coord}");
                if tile_coord
                    .path(&tile_settings)
                    .try_exists()
                    .unwrap_or(false)
                {
                    commands.spawn(TileBundle::from_tile_coord(
                        *tile_coord,
                        &server,
                        &tile_settings,
                    ));
                } else if !pending_tiles.contains_key(tile_coord) {
                    let url = tile_coord.url(&tile_settings);
                    let tile_coord = *tile_coord;
                    let path = tile_coord.path(&tile_settings);
                    let new_task = thread_pool.spawn(async move {
                        if std::env::var("NO_DOWNLOAD").is_ok() {
                            let col = if (tile_coord.x + tile_coord.y) % 2 == 0 {
                                150
                            } else {
                                200
                            };
                            RgbaImage::from_pixel(1, 1, Rgba::from([col, col, col, 255]))
                                .save_with_format(path, ImageFormat::Png)?;
                        } else {
                            let lock = SEMAPHORE.acquire().await;
                            let bytes = surf::get(url).recv_bytes().await?;
                            drop(lock);
                            async_fs::write(path, &bytes).await?;
                        };

                        Ok(())
                    });
                    pending_tiles.insert(tile_coord, new_task);
                }
            }
        }
    }

    let mut to_remove = vec![];
    for (tile_coord, task) in pending_tiles.iter_mut() {
        if !shown_tiles.contains(tile_coord) {
            to_remove.push((*tile_coord, true));
            continue;
        }
        if task.is_finished() {
            if matches!(future::block_on(task), Ok(())) {
                commands.spawn(TileBundle::from_tile_coord(
                    *tile_coord,
                    &server,
                    &tile_settings,
                ));
            };
            to_remove.push((*tile_coord, false));
        }
    }
    for (remove, cancel) in to_remove {
        if let Some(a) = pending_tiles.remove(&remove) {
            if cancel {
                thread_pool.spawn(a.cancel()).detach();
            }
        }
    }
    server.free_unused_assets();
}
