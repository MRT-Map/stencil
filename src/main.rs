#![windows_subsystem = "windows"]

use bevy::{
    asset::AssetPlugin, diagnostic::FrameTimeDiagnosticsPlugin, log::LogSettings, prelude::*,
    render::texture::ImageSettings, window::WindowMode,
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
        .insert_resource(WindowDescriptor {
            title: "Stencil".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        })
        .insert_resource(LogSettings {
            filter: "warn,bevy_asset::asset_server=error,surf::middleware::logger::native=off,isahc::handler=error,stencil2=debug".into(),
            level: bevy::log::Level::DEBUG,
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<AssetPlugin, _>(WebAssetPlugin)
        })
        .add_plugins(DefaultPickingPlugins)
        //.add_plugin(LogDiagnosticsPlugin::default())
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
