#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use dirs_paths::data_dir;
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use ui::tilemap::RenderingPlugin;

#[cfg(target_os = "linux")]
use crate::window::settings::LinuxWindow;
use crate::{
    component::{
        actions::ComponentActionPlugins, panels::ComponentPanelsPlugin, tools::ComponentToolPlugins,
    },
    history::HistoryPlugin,
    info_windows::InfoWindowsPlugin,
    init::InitPlugin,
    keymaps::KeymapPlugin,
    misc_config::MiscSettingsPlugin,
    project::ProjectPlugin,
    ui::{notif::NotifPlugin, UiPlugin},
    window::{settings::INIT_WINDOW_SETTINGS, WindowSettingsPlugin},
};
#[cfg(debug_assertions)]
use crate::inspector::InspectorPlugin;

pub mod component;
pub mod dirs_paths;
pub mod file;
pub mod history;
pub mod info_windows;
pub mod init;
pub mod keymaps;
pub mod misc_config;
pub mod panic;
pub mod project;
pub mod state;
pub mod tile;
pub mod ui;
pub mod window;
#[cfg(debug_assertions)]
pub mod inspector;

fn init_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().compact().with_writer(
                std::io::stdout
                    .with_max_level(Level::DEBUG)
                    .and(tracing_appender::rolling::hourly(data_dir("logs"), "log")),
            ),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::try_new(
                "info,\
            wgpu_core::device=warn,\
            bevy_asset::asset_server=error,\
            surf::middleware::logger::native=off,\
            isahc::handler=error,\
            stencil2=debug",
            )
            .unwrap()
        }))
        .with(ErrorLayer::default())
        .init();
}

fn main() {
    std::panic::set_hook(Box::new(panic::panic));

    init_logger();
    info!("Logger initialised");

    #[cfg(target_os = "linux")]
    unsafe {
        match INIT_WINDOW_SETTINGS.display_server_protocol {
            LinuxWindow::Xorg => std::env::set_var("WINIT_UNIX_BACKEND", "x11"),
            LinuxWindow::Wayland => std::env::set_var("WINIT_UNIX_BACKEND", "wayland"),
            LinuxWindow::Auto => (),
        }
    }

    let mut app = App::new();
    app.add_plugins({
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Stencil".into(),
                    mode: INIT_WINDOW_SETTINGS.window_mode,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                file_path: data_dir("assets").to_string_lossy().to_string(),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: Some(INIT_WINDOW_SETTINGS.backends.into()),
                    ..default()
                }),
                ..default()
            })
            .disable::<LogPlugin>()
    })
    .add_plugins(FrameTimeDiagnosticsPlugin);

    app.add_plugins(MeshPickingPlugin)
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            ..default()
        })
        .add_plugins(EguiPlugin)
        .add_plugins(ShapePlugin);

    app.add_plugins(InitPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(RenderingPlugin)
        .add_plugins(ComponentToolPlugins)
        .add_plugins(ComponentActionPlugins)
        .add_plugins(InfoWindowsPlugin)
        .add_plugins(KeymapPlugin)
        .add_plugins(WindowSettingsPlugin)
        .add_plugins(ProjectPlugin)
        .add_plugins(HistoryPlugin)
        .add_plugins(NotifPlugin)
        .add_plugins(MiscSettingsPlugin)
        .add_plugins(ComponentPanelsPlugin);

    #[cfg(debug_assertions)]
    app.add_plugins(InspectorPlugin);

    app.run();
}
