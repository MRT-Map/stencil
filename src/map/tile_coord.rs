use std::{
    fmt::{Display, Formatter},
    num::NonZeroUsize,
    path::PathBuf,
    sync::{LazyLock, Mutex, MutexGuard},
};

use async_executor::Task;
use futures_lite::future;
use itertools::Either;
use lru::LruCache;
use tracing::error;

use crate::{EXECUTOR, map::basemap::Basemap};

#[derive(Default, PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
    pub z: i8,
}

impl Display for TileCoord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.z, self.x, self.y)
    }
}

impl TileCoord {
    pub const fn new(z: i8, x: i32, y: i32) -> Self {
        Self { x, y, z }
    }
    pub fn at_world_coord(coord: geo::Coord<f32>, z: i8, basemap: &Basemap) -> Self {
        Self {
            x: ((coord.x + basemap.offset.0) / basemap.tile_world_size(z)).floor() as i32,
            y: ((coord.y + basemap.offset.1) / basemap.tile_world_size(z)).floor() as i32,
            z,
        }
    }
    pub fn world_top_left(self, basemap: &Basemap) -> geo::Coord<f32> {
        geo::coord! {
            x: (self.x as f32).mul_add(basemap.tile_world_size(self.z), -basemap.offset.0),
            y: (self.y as f32).mul_add(basemap.tile_world_size(self.z), -basemap.offset.1),
        }
    }
    pub fn cache_path(self, basemap: &Basemap) -> PathBuf {
        let path = basemap
            .cache_path()
            .join(self.z.to_string())
            .join(self.x.to_string());
        let _ = std::fs::create_dir_all(&path);
        path.join(format!("{}.{}", self.y, basemap.extension))
    }
    pub fn texture_id(
        self,
        ctx: &egui::Context,
        basemap: &Basemap,
        tile_cache: &mut MutexGuard<LruCache<Self, TileCacheItem>>,
    ) -> Option<TextureIdResult> {
        let url = basemap.url(self);
        let item = tile_cache.get_or_insert_mut(self, || {
            let cache_path = self.cache_path(basemap);
            if cache_path.exists()
                && let Ok(a) = std::fs::read(cache_path)
            {
                return TileCacheItem::Loaded(Ok(a));
            }

            TileCacheItem::Pending(EXECUTOR.spawn(async move { surf::get(url).recv_bytes().await }))
        });
        let item_result = match item {
            TileCacheItem::Pending(task) => match future::block_on(future::poll_once(task)) {
                None => {
                    ctx.request_repaint_after_secs(0.25);
                    Either::Right(true)
                }
                Some(Ok(bytes)) => {
                    let cache_path = self.cache_path(basemap);
                    let _ = std::fs::write(cache_path, &bytes).map_err(|a| error!("{a:?}"));

                    *item = TileCacheItem::Loaded(Ok(bytes.clone()));
                    Either::Left(bytes)
                }
                Some(Err(e)) => {
                    *item = TileCacheItem::Loaded(Err(e));
                    Either::Right(false)
                }
            },
            TileCacheItem::Loaded(Ok(bytes)) => Either::Left(bytes.clone()),
            TileCacheItem::Loaded(Err(_)) => Either::Right(false),
        };
        match item_result {
            Either::Left(bytes) => {
                let cache_path = self.cache_path(basemap);
                let poll = egui::ImageSource::Bytes {
                    uri: format!("bytes://{}", cache_path.display()).into(),
                    bytes: bytes.into(),
                }
                .load(
                    ctx,
                    egui::TextureOptions::LINEAR,
                    egui::SizeHint::Scale(2.0.into()),
                )
                .ok()?;
                poll.texture_id().map(TextureIdResult::Success)
            }
            Either::Right(still_loading) => still_loading.then_some(TextureIdResult::Loading),
        }
    }
}

pub enum TileCacheItem {
    Pending(Task<surf::Result<Vec<u8>>>),
    Loaded(surf::Result<Vec<u8>>),
}
pub enum TextureIdResult {
    Success(egui::TextureId),
    Loading,
}
pub static TILE_CACHE: LazyLock<Mutex<LruCache<TileCoord, TileCacheItem>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(0x1_0000).unwrap())));
