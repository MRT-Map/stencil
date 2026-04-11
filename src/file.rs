use std::{
    any::Any,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::SystemTime,
};

use egui_notify::ToastLevel;
use etcetera::{AppStrategyArgs, app_strategy};
use eyre::Result;
use tracing::debug;

use crate::ui::notif::NotifState;

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

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        pub type AppStrategy = Dev;
    } else if #[cfg(target_os = "windows")] {
        pub type AppStrategy = app_strategy::Windows;
    } else if #[cfg(any(target_os = "macos", target_os = "ios"))] {
        pub type AppStrategy = app_strategy::Apple;
    } else {
        pub type AppStrategy = app_strategy::Xdg;
    }
}

pub static FOLDERS: LazyLock<AppStrategy> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    return Dev;
    #[cfg(not(debug_assertions))]
    app_strategy::choose_native_strategy(AppStrategyArgs {
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
    std::env::temp_dir().join("stencil3");
});

pub fn safe_write<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
    notifs: &mut NotifState,
) -> std::io::Result<()> {
    let _ = safe_delete(&path, notifs);
    if let Some(folder) = path.as_ref().parent() {
        let _ = std::fs::create_dir_all(folder);
    }
    std::fs::write(path, contents)
}
pub fn safe_delete<T: AsRef<Path>>(path: T, notifs: &mut NotifState) -> Result<Option<PathBuf>> {
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
            notifs.push_error(
                format!("Could not safe delete file/directory {}", path.display()),
                &e,
                ToastLevel::Warning,
            );
            Err(e.into())
        }
    }
}
