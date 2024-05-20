use std::{
    any::Any,
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::prelude::Event;
use egui_notify::ToastLevel;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::log::{ErrorLogEntry, ERROR_LOG};

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

#[must_use]
pub fn load_file<
    T: DeserializeOwned,
    F: for<'a> FnOnce(&'a Path) -> std::io::Result<Result<T, E>> + 'static,
    E: serde::de::Error,
>(
    file: &Path,
    deserializer: F,
    error: Option<&'static str>,
) -> Option<T> {
    match deserializer(file) {
        Ok(Ok(o)) => return Some(o),
        Ok(Err(e)) => {
            if let Some(thing) = error {
                let mut error_log = ERROR_LOG.write().unwrap();
                error_log.pending_errors.push(ErrorLogEntry::new(
                    &format!(
                        "Could not parse {thing} file {}:\n{e}",
                        file.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                ));
            }
        }
        Err(e) => {
            if let Some(thing) = error {
                let mut error_log = ERROR_LOG.write().unwrap();
                error_log.pending_errors.push(ErrorLogEntry::new(
                    &format!(
                        "Could not load {thing} file {}:\n{e}",
                        file.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                ));
            }
        }
    }
    None
}

#[must_use]
pub fn load_toml<T: DeserializeOwned>(file: &Path, error: Option<&'static str>) -> Option<T> {
    load_file(
        file,
        |file| std::fs::read_to_string(file).map(|c| toml::from_str(&c)),
        error,
    )
}

#[must_use]
pub fn load_msgpack<T: DeserializeOwned>(file: &Path, error: Option<&'static str>) -> Option<T> {
    load_file(
        file,
        |file| std::fs::read(file).map(|c| rmp_serde::from_slice(&c)),
        error,
    )
}

#[must_use]
pub fn save_file<
    T: Serialize,
    F: FnOnce(&T) -> Result<A, E>,
    A: AsRef<[u8]>,
    E: serde::ser::Error,
>(
    o: &T,
    serializer: F,
    file: &Path,
    error: Option<&'static str>,
) -> bool {
    match serializer(o).map(move |s| std::fs::write(file, s)) {
        Ok(Ok(_)) => return true,
        Ok(Err(e)) => {
            if let Some(thing) = error {
                let mut error_log = ERROR_LOG.write().unwrap();
                error_log.pending_errors.push(ErrorLogEntry::new(
                    &format!(
                        "Could not write {thing} file {}:\n{e}",
                        file.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                ));
            }
        }
        Err(e) => {
            if let Some(thing) = error {
                let mut error_log = ERROR_LOG.write().unwrap();
                error_log.pending_errors.push(ErrorLogEntry::new(
                    &format!(
                        "Could not serialise {thing} file {}:\n{e}",
                        file.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                ));
            }
        }
    }
    false
}

#[must_use]
pub fn save_toml<T: Serialize>(o: &T, file: &Path, error: Option<&'static str>) -> bool {
    save_file(o, |o| toml::to_string_pretty(o), file, error)
}

#[must_use]
pub fn save_msgpack<T: Serialize>(o: &T, file: &Path, error: Option<&'static str>) -> bool {
    save_file(o, |o| rmp_serde::to_vec_named(o), file, error)
}
