mod tile;
mod utils;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_mouse_tracking_plugin::{MainCamera, MousePosPlugin};
use bevy_web_asset::WebAssetPlugin;
use crate::tile::*;
use crate::utils::get_cursor_world_pos;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<bevy::asset::AssetPlugin, _>(WebAssetPlugin)
        })
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(world_pos)

        .add_plugin(MousePosPlugin::SingleCamera)
        .add_startup_system(setup)

        .insert_resource(Zoom(7.0))
        .add_system(show_tiles)
        .add_system(mouse_drag)
        .add_system(mouse_zoom)
        .run();
}

fn world_pos(
    windows: Res<Windows>,
    mut q_camera: Query<(&Camera, &mut GlobalTransform), With<MainCamera>>,
    mut texts: Query<&mut Text, With<CursorCoords>>
) {
    let (camera, transform) = q_camera.single_mut();
    let cursor_pos = get_cursor_world_pos(&windows, camera, &transform);

    if let Some(cursor_pos) = cursor_pos {
        let mut text = texts.single_mut();
        text.sections[0].value = format!("x: {} z: {}", cursor_pos.x.round(), cursor_pos.y.round());
    }
}

#[derive(Component)]
struct CursorCoords;

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(MainCamera);
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            "x: ? z: ?",
            TextStyle {
                font: server.load("NotoSans-Medium.ttf"),
                font_size: 20.0,
                color: Color::WHITE
            },
            TextAlignment { horizontal: HorizontalAlign::Right, ..default() }
        ),
        style: Style {
            position: Rect {
                top: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            position_type: PositionType::Absolute,

            ..default()
        },
        ..default()
    }).insert(CursorCoords);
}