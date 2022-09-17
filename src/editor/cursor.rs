use bevy::{prelude::*, render::camera::RenderTarget, sprite::Anchor};
use bevy_mouse_tracking_plugin::MainCamera;
use iyes_loopless::prelude::*;

use crate::{
    editor::{selecting_component::HoveringOverComponent, ui::HoveringOverGui},
    types::{EditorState, zoom::Zoom},
};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Crosshair;

#[tracing::instrument(skip_all)]
pub fn crosshair(
    mut commands: Commands,
    state: Res<CurrentState<EditorState>>,
    mut ch: Query<(Entity, &mut Transform, &mut Sprite), With<Crosshair>>,
    mut q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    server: Res<AssetServer>,
    windows: Res<Windows>,
    zoom: Res<Zoom>,
) {
    if !matches!(state.0, EditorState::CreatingComponent(_)) {
        for (e, _, _) in ch.iter() {
            debug!("Despawning crosshair");
            commands.entity(e).despawn();
        }
        return;
    }
    let (camera, c_transform) = q_camera.single_mut();
    let mouse_world_pos = if let Some(p) = get_cursor_world_pos(&windows, camera, c_transform) {
        p
    } else {
        for (e, _, _) in ch.iter() {
            debug!("Despawning crosshair");
            commands.entity(e).despawn();
        }
        return;
    };
    let new_transform = Transform::from_translation(mouse_world_pos.round().extend(100.0));
    let new_custom_size = Some(Vec2::splat(2f32.powf(8f32 - zoom.0) * 16f32));
    if ch.is_empty() {
        debug!("Spawning crosshair");
        commands
            .spawn_bundle(SpriteBundle {
                texture: server.load("crosshair.png"),
                transform: new_transform,
                sprite: Sprite {
                    custom_size: new_custom_size,
                    anchor: Anchor::Center,
                    ..default()
                },
                ..default()
            })
            .insert(Crosshair);
    } else {
        trace!("Updating crosshair location");
        let (_, mut transform, mut sprite) = ch.single_mut();
        *transform = new_transform;
        sprite.custom_size = new_custom_size;
    }
}

#[tracing::instrument(skip_all)]
pub fn cursor_icon(
    buttons: Res<Input<MouseButton>>,
    mut windows: ResMut<Windows>,
    state: Res<CurrentState<EditorState>>,
    hovering_over_gui: Res<HoveringOverGui>,
    hovering_over_comp: Res<HoveringOverComponent>,
) {
    if !hovering_over_gui.0 {
        if matches!(state.0, EditorState::CreatingComponent(_)) {
            windows.primary_mut().set_cursor_visibility(false);
            return;
        } else {
            windows.primary_mut().set_cursor_visibility(true);
        }
        windows.primary_mut().set_cursor_icon(match state.0 {
            EditorState::Loading => CursorIcon::Wait,
            EditorState::Idle | EditorState::DeletingComponent => {
                if hovering_over_comp.0 {
                    CursorIcon::Hand
                } else if buttons.pressed(MouseButton::Left) {
                    CursorIcon::Grabbing
                } else {
                    CursorIcon::Grab
                }
            }
            EditorState::CreatingComponent(_) => unreachable!(),
            EditorState::EditingNodes => CursorIcon::Hand,
            EditorState::MovingComponent => CursorIcon::Hand,
            EditorState::RotatingComponent => CursorIcon::Hand,
        });
    } else {
        windows.primary_mut().set_cursor_visibility(true);
    }
}

pub fn world_pos(
    windows: Res<Windows>,
    mut q_camera: Query<(&Camera, &mut GlobalTransform), With<MainCamera>>,
    mut texts: Query<&mut Text, With<CursorCoords>>,
) {
    let (camera, transform) = q_camera.single_mut();
    let cursor_pos = get_cursor_world_pos(&windows, camera, &transform);

    if let Some(cursor_pos) = cursor_pos {
        let mut text = texts.single_mut();
        text.sections[0].value = format!("x: {} z: {}", cursor_pos.x.round(), cursor_pos.y.round());
    }
}

#[derive(Component)]
pub struct CursorCoords;

pub fn cursor_setup(mut commands: Commands, server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "x: ? z: ?",
                TextStyle {
                    font: server.load("NotoSans-Medium.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER_RIGHT),
            style: Style {
                position: UiRect {
                    top: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .insert(CursorCoords);
}

pub fn get_cursor_world_pos(
    windows: &Res<Windows>,
    camera: &Camera,
    transform: &GlobalTransform,
) -> Option<Vec2> {
    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id)?
    } else {
        windows.get_primary()?
    };

    if let Some(screen_pos) = wnd.cursor_position() {
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = transform.compute_matrix() * camera.projection_matrix().inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        Some(world_pos.truncate())
    } else {
        None
    }
}

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .with_system(cursor_icon)
                .with_system(crosshair)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .with_system(world_pos)
                .into(),
        )
        .add_exit_system(EditorState::Loading, cursor_setup);
    }
}
