use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
    time::SystemTime,
};

use egui_notify::ToastLevel;
use eyre::Result;
use tracing::debug;

use crate::ui::notif::NotifState;

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("data");
    #[cfg(not(debug_assertions))]
    let dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stencil3");
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

pub static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("cache");
    #[cfg(not(debug_assertions))]
    let dir = dirs::cache_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stencil3");
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

pub fn safe_write<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
    notifs: &mut NotifState,
) -> std::io::Result<()> {
    let _ = safe_delete(&path, notifs);
    std::fs::write(path, contents)
}
pub fn safe_delete<T: AsRef<Path>>(path: T, notifs: &mut NotifState) -> Result<Option<PathBuf>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    let trash_dir = cache_dir("trash");
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let new_path = trash_dir.join(format!(
        "{timestamp}-{}",
        path.file_name().unwrap_or_default().display()
    ));
    match std::fs::rename(path, &new_path) {
        Ok(()) => {
            debug!("Safe deleted {}", path.display());
            Ok(Some(new_path))
        }
        Err(e) => {
            notifs.push_error(
                format!("Could not safe delete file/directory {}", path.display()),
                &e,
                ToastLevel::Warning,
            );
            Err(e.into())
        }
    }
}
