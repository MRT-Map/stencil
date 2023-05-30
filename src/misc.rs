use std::{
    any::Any,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;

pub static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stencil2");
    let _ = std::fs::create_dir_all(&dir);
    dir
});

pub fn data_dir(next: impl AsRef<Path>) -> PathBuf {
    let path = DATA_DIR.join(next);
    let _ = std::fs::create_dir_all(&path);
    path
}

pub fn data_file(next: impl AsRef<Path>) -> PathBuf {
    DATA_DIR.join(next)
}

pub type Action = Box<dyn Any + Send + Sync>;
