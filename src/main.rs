mod editor;
mod rendering;
mod types;

use bevy::asset::AssetPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mouse_tracking_plugin::{prelude::*, MainCamera};
use bevy_prototype_lyon::prelude::*;
use bevy_web_asset::WebAssetPlugin;
use iyes_loopless::prelude::*;
use rendering::{
    mouse_nav::{mouse_drag, mouse_zoom},
    tile::*,
    utils::*,
};
use types::zoom::Zoom;

use crate::{
    editor::{creating_component::CreateComponentPlugin, cursor::CursorPlugin, UiPlugin},
    pla::{PlaComponent, PlaNode},
    rendering::RenderingPlugin,
    skin::{get_skin, Skin},
    types::*,
};

pub struct LoadPlugin;
impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(EditorState::Loading)
            .init_resource::<Skin>()
            .init_resource::<Vec<PlaComponent>>()
            .init_resource::<Vec<PlaNode>>()
            .add_startup_system(get_skin)
            .add_exit_system(EditorState::Loading, setup);
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<AssetPlugin, _>(WebAssetPlugin)
        })
        //.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LoadPlugin)
        .add_plugin(MousePosPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(UiPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(RenderingPlugin)
        .add_plugin(CreateComponentPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera)
        .insert(UiCameraConfig { show_ui: true });
}
