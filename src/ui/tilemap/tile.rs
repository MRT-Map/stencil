use std::{collections::HashMap, sync::LazyLock};

use async_executor::{Executor, Task};
use async_lock::Semaphore;
use bevy::{ecs::query::QueryFilter, prelude::*};
use futures_lite::future;
use image::{ImageFormat, Rgba, RgbaImage};

use crate::{
    tile::{
        bundle::{Tile, TileBundle},
        tile_coord::TileCoord,
        utils::get_map_coords_of_edges,
        zoom::Zoom,
    },
    ui::tilemap::settings::{Basemap, TileSettings, INIT_TILE_SETTINGS},
};

static SEMAPHORE: LazyLock<Semaphore> =
    LazyLock::new(|| Semaphore::new(INIT_TILE_SETTINGS.max_get_requests));

#[must_use]
pub fn get_shown_tiles<R: QueryFilter>(
    q_camera: &Query<(&Camera, Ref<Transform>), R>,
    zoom: i8,
    basemap: &Basemap,
) -> Result<Vec<TileCoord>> {
    let (camera, transform) = q_camera.single()?;
    let (c_left, c_top, c_right, c_bottom) = get_map_coords_of_edges(camera, &transform);
    let TileCoord {
        x: t_left,
        y: t_top,
        ..
    } = TileCoord::from_world_coords(
        f64::from(c_left),
        f64::from(c_top),
        zoom.min(basemap.max_tile_zoom),
        basemap,
    );
    let TileCoord {
        x: t_right,
        y: t_bottom,
        ..
    } = TileCoord::from_world_coords(
        f64::from(c_right),
        f64::from(c_bottom),
        zoom.min(basemap.max_tile_zoom),
        basemap,
    );

    Ok((t_left - 1..=t_right + 1)
        .flat_map(|x| {
            (t_top - 1..=t_bottom + 1)
                .map(|y| TileCoord {
                    x,
                    y,
                    z: zoom.min(basemap.max_tile_zoom),
                })
                .collect::<Vec<_>>()
        })
        .collect())
}

#[derive(Resource, Default)]
pub struct PendingTiles(pub HashMap<TileCoord, Task<surf::Result<()>>>);

#[tracing::instrument(skip_all)]
pub fn show_tiles_sy(
    mut commands: Commands,
    q_camera: Query<(&Camera, Ref<Transform>)>,
    mut query: Query<(Entity, &TileCoord), With<Tile>>,
    zoom: Res<Zoom>,
    server: Res<AssetServer>,
    tile_settings: Res<TileSettings>,
    mut pending_tiles: ResMut<PendingTiles>,
    mut old_basemap: Local<Basemap>,
    mut executor: Local<Option<Executor>>,
) -> Result {
    if q_camera.is_empty() {
        return Ok(());
    }
    let basemap = &tile_settings.basemaps[0];
    if *basemap != *old_basemap {
        for (e, _) in &query {
            commands.entity(e).despawn();
        }
        pending_tiles.0.clear();
    }
    basemap.clone_into(&mut old_basemap);

    let (camera, transform) = q_camera.single()?;
    let mut shown_tiles = get_shown_tiles(&q_camera, zoom.0.round() as i8, basemap)?;
    let executor = executor.get_or_insert_with(Executor::new);
    executor.try_tick();
    if !transform.is_changed() {
        let (ml, mt, mr, mb) = get_map_coords_of_edges(camera, &transform);
        for (e, tile_coord) in &mut query {
            if (zoom.0 <= f32::from(basemap.max_tile_zoom) && tile_coord.z > zoom.0.round() as i8)
                || (zoom.0 > f32::from(basemap.max_tile_zoom)
                    && tile_coord.z != basemap.max_tile_zoom)
                || (zoom.0 > f32::from(basemap.max_tile_zoom) && {
                    let (tl, tt, tr, tb) = tile_coord.get_edges(basemap);
                    tr < ml || tl > mr || tb < mt || tt > mb
                })
                || (tile_coord.z <= (basemap.max_tile_zoom - 1)
                    && zoom.0 <= f32::from(basemap.max_tile_zoom)
                    && !shown_tiles.contains(tile_coord))
            {
                trace!("Hiding {tile_coord}");
                commands.entity(e).despawn();
            } else {
                shown_tiles.retain(|t| t != tile_coord);
                trace!("Showing {tile_coord}");
            }
        }
        for tile_coord in &shown_tiles {
            if tile_coord.z <= basemap.max_tile_zoom {
                trace!("Loading tile {tile_coord}");
                if tile_coord.path(basemap).try_exists().unwrap_or(false) {
                    commands.spawn(TileBundle::from_tile_coord(*tile_coord, &server, basemap));
                } else if !pending_tiles.0.contains_key(tile_coord) {
                    let url = tile_coord.url(basemap);
                    let tile_coord = *tile_coord;
                    let path = tile_coord.path(basemap);
                    let extension = basemap.extension.clone();
                    let new_task = executor.spawn(async move {
                        if std::env::var("NO_DOWNLOAD").is_ok() {
                            let col = if (tile_coord.x + tile_coord.y) % 2 == 0 {
                                150
                            } else {
                                200
                            };
                            RgbaImage::from_pixel(1, 1, Rgba::from([col, col, col, 255]))
                                .save_with_format(
                                    path,
                                    ImageFormat::from_extension(extension)
                                        .unwrap_or(ImageFormat::Png),
                                )?;
                        } else {
                            let lock = SEMAPHORE.acquire().await;
                            let mut response = surf::get(url).await?;
                            drop(lock);
                            if !response.status().is_server_error() {
                                async_fs::write(path, response.body_bytes().await?).await?;
                            }
                        }

                        Ok(())
                    });
                    pending_tiles.0.insert(tile_coord, new_task);
                }
            }
        }
    }

    let mut to_remove = vec![];
    for (tile_coord, task) in &mut pending_tiles.0 {
        executor.try_tick();
        if !shown_tiles.contains(tile_coord) {
            to_remove.push((*tile_coord, true));
            continue;
        }
        if task.is_finished() {
            if matches!(future::block_on(task), Ok(())) {
                commands.spawn(TileBundle::from_tile_coord(*tile_coord, &server, basemap));
            }
            to_remove.push((*tile_coord, false));
        }
    }
    for (remove, cancel) in to_remove {
        if let Some(a) = pending_tiles.0.remove(&remove) {
            if cancel {
                executor.spawn(a.cancel()).detach();
            }
        }
    }
    //server.free_unused_assets();
    Ok(())
}
