use bevy::{
    asset::AssetPlugin, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*,
    render::texture::ImageSettings, window::WindowMode,
};
use bevy_egui::EguiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_mouse_tracking_plugin::prelude::MousePosPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_web_asset::WebAssetPlugin;
use editor::component_actions::moving::MoveComponentPlugin;

use crate::{
    editor::{
        component_actions::selecting::SelectComponentPlugin,
        component_tools::{
            creating::CreateComponentPlugin, deleting::DeleteComponentPlugin,
            node_editing::EditNodePlugin,
        },
        cursor::{mouse_events::MouseEventsPlugin, CursorPlugin},
        menu_actions::MenuPlugin,
        tilemap::RenderingPlugin,
        ui::UiPlugin,
    },
    setup::SetupPlugin,
};

mod editor;
mod setup;
mod types;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Stencil".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..default()
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
        .add_plugin(MouseEventsPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(RenderingPlugin)
        .add_plugin(CreateComponentPlugin)
        .add_plugin(SelectComponentPlugin)
        .add_plugin(DeleteComponentPlugin)
        .add_plugin(MoveComponentPlugin)
        .add_plugin(EditNodePlugin)
        .add_plugin(MenuPlugin)
        .run();
}
