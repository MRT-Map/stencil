use std::path::PathBuf;

use egui_notify::ToastLevel;
use eyre::Result;
use tracing::{debug, info};

use crate::{file::safe_write, ui::notif::NotifState};

#[macro_export]
macro_rules! impl_load_save {
    (toml $t:ty, $path:expr) => {
        impl $crate::load_save::LoadSave for $t {
            fn path() -> std::path::PathBuf {
                $path
            }
            fn ser(&self) -> eyre::Result<Vec<u8>> {
                toml::to_string_pretty(&self)
                    .map(|a| a.into_bytes())
                    .map_err(|e| e.into())
            }
            fn de(ser: &[u8]) -> eyre::Result<Self> {
                toml::from_slice(ser).map_err(|e| e.into())
            }
        }
    };
    (toml $t:ty, $path:expr, $header:expr) => {
        impl $crate::load_save::LoadSave for $t {
            fn path() -> std::path::PathBuf {
                $path
            }
            fn ser(&self) -> eyre::Result<Vec<u8>> {
                toml::to_string_pretty(&self)
                    .map(|a| format!("{}\n\n{a}", $header).into_bytes())
                    .map_err(|e| e.into())
            }
            fn de(ser: &[u8]) -> eyre::Result<Self> {
                toml::from_slice(ser).map_err(|e| e.into())
            }
        }
    };
    (mpk $t:ty, $path:expr) => {
        impl $crate::load_save::LoadSave for $t {
            fn path() -> std::path::PathBuf {
                $path
            }
            fn ser(&self) -> eyre::Result<Vec<u8>> {
                rmp_serde::to_vec_named(self).map_err(|e| e.into())
            }
            fn de(ser: &[u8]) -> eyre::Result<Self> {
                rmp_serde::from_slice(ser).map_err(|e| e.into())
            }
        }
    };
}

pub trait LoadSave: Default {
    fn path() -> PathBuf;
    fn ser(&self) -> Result<Vec<u8>>;
    fn de(ser: &[u8]) -> Result<Self>;

    fn load(notifs: &mut NotifState) -> Self {
        if !Self::path().exists() {
            info!("Loading default file for {}", Self::path().display());
            let s = Self::default();
            let () = s.save(notifs);
            return s;
        }

        let vec = match std::fs::read(Self::path()) {
            Ok(vec) => {
                info!("Read file at {}", Self::path().display());
                vec
            }
            Err(e) => {
                notifs.push_error(
                    format!("Couldn't open file at {}", Self::path().display()),
                    e,
                    ToastLevel::Error,
                );
                return Self::default();
            }
        };

        match Self::de(&vec) {
            Ok(s) => {
                info!("Deserialised file at {}", Self::path().display());
                s
            }
            Err(e) => {
                notifs.push_error(
                    format!("Couldn't deserialise file at {}", Self::path().display()),
                    e,
                    ToastLevel::Error,
                );
                Self::default()
            }
        }
    }
    fn save(&self, notifs: &mut NotifState) {
        let vec = match self.ser() {
            Ok(vec) => {
                debug!("Serialised file for {}", Self::path().display());
                vec
            }
            Err(e) => {
                notifs.push_error(
                    format!("Couldn't serialise file for {}", Self::path().display()),
                    e,
                    ToastLevel::Error,
                );
                return;
            }
        };

        match safe_write(Self::path(), vec, notifs) {
            Ok(()) => {
                debug!("Wrote file at {}", Self::path().display());
            }
            Err(e) => {
                notifs.push_error(
                    format!("Couldn't write file for {}", Self::path().display()),
                    e,
                    ToastLevel::Error,
                );
            }
        }
    }
}
