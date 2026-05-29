use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
    time::SystemTime,
};

use etcetera::app_strategy;
use eyre::Result;
use tracing::debug;

use crate::notif;

#[cfg(debug_assertions)]
pub struct Dev;

#[cfg(debug_assertions)]
impl app_strategy::AppStrategy for Dev {
    fn home_dir(&self) -> &Path {
        unimplemented!()
    }
    fn config_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/config")
    }
    fn data_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/data")
    }
    fn cache_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/cache")
    }
    fn state_dir(&self) -> Option<PathBuf> {
        unimplemented!()
    }
    fn runtime_dir(&self) -> Option<PathBuf> {
        unimplemented!()
    }
}

cfg_select! {
    debug_assertions => {
        pub type AppStrategy = Dev;
    }
    target_os = "windows" => {
        pub type AppStrategy = app_strategy::Windows;
    }
    any(target_os = "macos", target_os = "ios") => {
        pub type AppStrategy = app_strategy::Apple;
    }
    _ => {
        pub type AppStrategy = app_strategy::Xdg;
    }
}

pub static FOLDERS: LazyLock<AppStrategy> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    return Dev;
    #[cfg(not(debug_assertions))]
    app_strategy::choose_native_strategy(etcetera::AppStrategyArgs {
        top_level_domain: "io.github".into(),
        author: "mrt-map".into(),
        app_name: "stencil3".into(),
    })
    .unwrap()
});

pub static TRASH_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/trash");
    #[cfg(not(debug_assertions))]
    std::env::temp_dir().join("stencil3")
});

pub fn safe_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
    let _ = safe_delete(&path);
    if let Some(folder) = path.as_ref().parent() {
        let _ = std::fs::create_dir_all(folder);
    }
    std::fs::write(path, contents)
}
pub fn safe_delete<T: AsRef<Path>>(path: T) -> Result<Option<PathBuf>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    let _ = std::fs::create_dir_all(&*TRASH_DIR);
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let new_path = TRASH_DIR.join(format!(
        "{timestamp}-{}",
        path.file_name().unwrap_or_default().display()
    ));
    match std::fs::rename(path, &new_path) {
        Ok(()) => {
            debug!("Safe deleted {}", path.display());
            Ok(Some(new_path))
        }
        Err(e) => {
            let errors = [&e];
            notif!(warning format!("Could not safe delete file/directory {}", path.display()), errors &errors);
            Err(e.into())
        }
    }
}
