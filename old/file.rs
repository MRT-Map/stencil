use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use egui_notify::ToastLevel;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    dirs_paths::cache_dir,
    ui::notif::{NOTIF_LOG, NotifLogRwLockExt},
};

pub fn load_file<
    T: DeserializeOwned,
    F: for<'a> FnOnce(&'a Path) -> std::io::Result<Result<T, E>> + 'static,
    E: serde::de::Error + Into<eyre::Report>,
>(
    file: &Path,
    deserializer: F,
    error: Option<&'static str>,
) -> eyre::Result<T> {
    match deserializer(file) {
        Ok(Ok(o)) => Ok(o),
        Ok(Err(e)) => {
            if let Some(thing) = error {
                NOTIF_LOG.push(
                    format!(
                        "Could not parse {thing} file {}:\n{e}",
                        file.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                );
            }
            Err(e.into())
        }
        Err(e) => {
            if let Some(thing) = error {
                NOTIF_LOG.push(
                    format!(
                        "Could not load {thing} file {}:\n{e}",
                        file.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                );
            }
            Err(e.into())
        }
    }
}

pub fn load_toml<T: DeserializeOwned>(file: &Path, error: Option<&'static str>) -> eyre::Result<T> {
    load_file(
        file,
        |file| std::fs::read_to_string(file).map(|c| toml::from_str(&c)),
        error,
    )
}

pub fn load_msgpack<T: DeserializeOwned>(
    file: &Path,
    error: Option<&'static str>,
) -> eyre::Result<T> {
    load_file(
        file,
        |file| std::fs::read(file).map(|c| rmp_serde::from_slice(&c)),
        error,
    )
}

pub fn save_file<
    T: Serialize,
    F: FnOnce(&T) -> Result<A, E>,
    A: AsRef<[u8]>,
    E: serde::ser::Error + Into<eyre::Report>,
>(
    o: &T,
    serializer: F,
    file: &Path,
    error: Option<&'static str>,
) -> eyre::Result<()> {
    let old_file = file.exists().then(|| safe_delete(file, None));

    match serializer(o).map(move |s| std::fs::write(file, s)) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => {
            if let Some(Ok(old_file)) = old_file {
                let _ = restore(&old_file, file, None);
            }
            if let Some(thing) = error {
                NOTIF_LOG.push(
                    format!(
                        "Could not write {thing} file {}:\n{e}",
                        file.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                );
            }
            Err(e.into())
        }
        Err(e) => {
            if let Some(thing) = error {
                NOTIF_LOG.push(
                    format!(
                        "Could not serialise {thing} file {}:\n{e}",
                        file.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                );
            }
            Err(e.into())
        }
    }
}

pub fn save_toml<T: Serialize>(
    o: &T,
    file: &Path,
    error: Option<&'static str>,
) -> eyre::Result<()> {
    save_file(o, |o| toml::to_string_pretty(o), file, error)
}

pub fn save_toml_with_header<T: Serialize>(
    o: &T,
    file: &Path,
    header: &'static str,
    error: Option<&'static str>,
) -> eyre::Result<()> {
    save_file(
        o,
        |o| toml::to_string_pretty(o).map(|a| format!("{header}\n\n{a}")),
        file,
        error,
    )
}

pub fn save_msgpack<T: Serialize>(
    o: &T,
    file: &Path,
    error: Option<&'static str>,
) -> eyre::Result<()> {
    save_file(o, |o| rmp_serde::to_vec_named(o), file, error)
}

pub fn safe_delete(path: &Path, error: Option<&'static str>) -> eyre::Result<PathBuf> {
    let trash_dir = cache_dir("trash");
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let new_path = trash_dir.join(timestamp.to_string());
    match std::fs::rename(path, &new_path) {
        Ok(()) => Ok(new_path),
        Err(e) => {
            if let Some(thing) = error {
                NOTIF_LOG.push(
                    format!(
                        "Could not safe delete {thing} file/directory {}:\n{e}",
                        path.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                );
            }
            Err(e.into())
        }
    }
}

pub fn restore(path: &Path, old_path: &Path, error: Option<&'static str>) -> eyre::Result<()> {
    match std::fs::rename(path, old_path) {
        Ok(()) => Ok(()),
        Err(e) => {
            if let Some(thing) = error {
                NOTIF_LOG.push(
                    format!(
                        "Could not restore {thing} from file/directory {}:\n{e}",
                        path.to_string_lossy()
                    ),
                    ToastLevel::Warning,
                );
            }
            Err(e.into())
        }
    }
}
