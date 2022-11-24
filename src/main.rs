#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{
    asset::AssetPlugin, diagnostic::FrameTimeDiagnosticsPlugin, log::LogPlugin, prelude::*,
    window::WindowMode,
};
use bevy_egui::EguiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_mouse_tracking_plugin::prelude::MousePosPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_web_asset::WebAssetPlugin;

use crate::{
    component_actions::ComponentActionPlugins, component_tools::ComponentToolPlugins,
    cursor::CursorPlugin, info_windows::InfoWindowsPlugin, load_save::LoadSavePlugin,
    setup::SetupPlugin, tilemap::RenderingPlugin, ui::UiPlugin,
};

mod component_actions;
mod component_tools;
mod cursor;
mod info_windows;
mod load_save;
mod misc;
mod pla2;
mod setup;
mod tilemap;
mod ui;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    App::new()
        .add_plugin(WebAssetPlugin {
            asset_plugin: AssetPlugin {
                //asset_folder: "".to_string(),
                ..default()
            }
        })
        .add_plugins({
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Stencil".to_string(),
                        mode: WindowMode::BorderlessFullscreen,
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    filter: "bevy_asset::asset_server=error,surf::middleware::logger::native=off,isahc::handler=error,stencil2=debug".into(),
                    level: bevy::log::Level::WARN,
                })
                .disable::<AssetPlugin>()
        })
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(MousePosPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(SetupPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(RenderingPlugin)
        .add_plugins(ComponentToolPlugins)
        .add_plugins(ComponentActionPlugins)
        .add_plugin(LoadSavePlugin)
        .add_plugin(InfoWindowsPlugin)
        .run();
}
