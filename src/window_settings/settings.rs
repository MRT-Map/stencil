use bevy::{prelude::*, render::settings::Backends, window::WindowMode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::misc::data_file;

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

impl From<SerializableBackends> for Backends {
    fn from(value: SerializableBackends) -> Self {
        let mut b = Backends::empty();
        if value.vulkan {
            b |= Backends::VULKAN;
        }
        if value.metal {
            b |= Backends::METAL;
        }
        if value.dx12 {
            b |= Backends::DX12;
        }
        if value.dx11 {
            b |= Backends::DX11;
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
            dx11: value.contains(Backends::DX11),
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

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Resource, Default)]
pub struct WindowSettings {
    #[serde(default, skip_serializing_if = "is_default")]
    pub backends: SerializableBackends,
    #[cfg(target_os = "linux")]
    #[serde(default, skip_serializing_if = "is_default")]
    pub display_server_protocol: LinuxWindow,
    #[serde(default, skip_serializing_if = "is_default")]
    pub window_mode: WindowMode,
}

pub static INIT_WINDOW_SETTINGS: Lazy<WindowSettings> =
    Lazy::new(
        || match std::fs::read(data_file("window_settings.msgpack")) {
            Ok(bytes) => {
                info!("Found window settings file");
                rmp_serde::from_slice(&bytes).unwrap()
            }
            Err(e) => {
                info!("Couldn't find or open window settings file: {e:?}");
                WindowSettings::default()
            }
        },
    );
