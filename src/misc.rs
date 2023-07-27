use std::{
    any::Any,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::prelude::Event;
use once_cell::sync::Lazy;

pub static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stencil2");
    let _ = std::fs::create_dir_all(&dir);
    dir
});

pub fn data_dir<T: AsRef<Path>>(next: T) -> PathBuf {
    let path = DATA_DIR.join(next);
    let _ = std::fs::create_dir_all(&path);
    path
}

pub fn data_path<T: AsRef<Path>>(next: T) -> PathBuf {
    DATA_DIR.join(next)
}

pub static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let dir = dirs::cache_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stencil2");
    let _ = std::fs::create_dir_all(&dir);
    dir
});

pub fn cache_dir<T: AsRef<Path>>(next: T) -> PathBuf {
    let path = CACHE_DIR.join(next);
    let _ = std::fs::create_dir_all(&path);
    path
}

pub fn cache_path<T: AsRef<Path>>(next: T) -> PathBuf {
    CACHE_DIR.join(next)
}

#[derive(Event, Clone)]
pub struct Action(Arc<dyn Any + Send + Sync>);
impl Action {
    pub fn new<T: Any + Send + Sync>(v: T) -> Self {
        Self(Arc::new(v))
    }
    #[must_use]
    pub fn downcast_ref<R: Any>(&self) -> Option<&R> {
        self.0.as_ref().downcast_ref()
    }
}
