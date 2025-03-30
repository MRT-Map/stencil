use std::path::{Path, PathBuf};

pub static DATA_DIR: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(|| {
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

pub static CACHE_DIR: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(|| {
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
