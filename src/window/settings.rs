use bevy::{prelude::*, render::settings::Backends, window::WindowMode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::log::AddToErrorLog, misc::data_path};

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Copy)]
pub struct SerializableBackends {
    pub vulkan: bool,
    pub metal: bool,
    pub dx12: bool,
    pub dx11: bool,
}

impl Default for SerializableBackends {
    fn default() -> Self {
        Self {
            vulkan: true,
            metal: true,
            dx11: true,
            dx12: true,
        }
    }
}

impl SerializableBackends {
    #[must_use]
    pub const fn is_none(self) -> bool {
        !(self.vulkan || self.metal || self.dx11 || self.dx12)
    }
}

impl From<SerializableBackends> for Backends {
    fn from(value: SerializableBackends) -> Self {
        let mut b = Self::empty();
        if value.vulkan {
            b |= Self::VULKAN;
        }
        if value.metal {
            b |= Self::METAL;
        }
        if value.dx12 {
            b |= Self::DX12;
        }
        b
    }
}

impl From<Backends> for SerializableBackends {
    fn from(value: Backends) -> Self {
        Self {
            vulkan: value.contains(Backends::VULKAN),
            metal: value.contains(Backends::METAL),
            dx12: value.contains(Backends::DX12),
            dx11: true,
        }
    }
}

#[cfg(target_os = "linux")]
#[derive(Deserialize, Serialize, Copy, Clone, Default, PartialEq, Eq)]
pub enum LinuxWindow {
    Wayland,
    Xorg,
    #[default]
    Auto,
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Resource, Default)]
pub struct WindowSettings {
    #[serde(default, skip_serializing_if = "is_default")]
    pub backends: SerializableBackends,
    #[cfg(target_os = "linux")]
    #[serde(default, skip_serializing_if = "is_default")]
    pub display_server_protocol: LinuxWindow,
    #[serde(default, skip_serializing_if = "is_default")]
    pub window_mode: WindowMode,
}

impl WindowSettings {
    pub fn load() -> color_eyre::Result<Self> {
        match std::fs::read_to_string(data_path("window_settings.toml")) {
            Ok(str) => {
                info!("Found window settings file");
                Ok(toml::from_str(&str)?)
            }
            Err(e) => {
                info!("Couldn't find or open window settings file: {e:?}");
                let s = Self::default();
                let _ = s.save();
                Ok(s)
            }
        }
    }
    pub fn save(&self) -> color_eyre::Result<()> {
        info!("Saving window settings file");
        let prefix_text = "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#window_settingstoml";
        let serialized = toml::to_string_pretty(self)?;

        Ok(std::fs::write(
            data_path("window_settings.toml"),
            format!("{prefix_text}\n\n{serialized}"),
        )?)
    }
}

pub static INIT_WINDOW_SETTINGS: Lazy<WindowSettings> =
    Lazy::new(|| WindowSettings::load().unwrap_or_default_and_log());
