use bevy::{math::Vec3Swizzles, prelude::*, sprite::Anchor};
use bevy_mouse_tracking_plugin::MousePosWorld;
use iyes_loopless::prelude::*;

use crate::{
    editor::{actions::mouse_events::HoveredComponent, ui::HoveringOverGui},
    types::{zoom::Zoom, EditorState},
};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Crosshair;

#[tracing::instrument(skip_all)]
pub fn crosshair_sy(
    mut commands: Commands,
    state: Res<CurrentState<EditorState>>,
    mut ch: Query<(Entity, &mut Transform, &mut Sprite), With<Crosshair>>,
    server: Res<AssetServer>,
    zoom: Res<Zoom>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    if !matches!(state.0, EditorState::CreatingComponent(_)) {
        for (e, _, _) in ch.iter() {
            debug!("Despawning crosshair");
            commands.entity(e).despawn();
        }
        return;
    }
    let new_transform = Transform::from_translation(mouse_pos_world.round().xy().extend(100.0));
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
pub fn cursor_icon_sy(
    buttons: Res<Input<MouseButton>>,
    mut windows: ResMut<Windows>,
    state: Res<CurrentState<EditorState>>,
    hovering_over_gui: Res<HoveringOverGui>,
    hovered_comp: Query<(), With<HoveredComponent>>,
) {
    if matches!(state.0, EditorState::CreatingComponent(_)) {
        windows
            .primary_mut()
            .set_cursor_visibility(hovering_over_gui.0);
        return;
    } else {
        windows.primary_mut().set_cursor_visibility(true);
        if hovering_over_gui.0 {
            return;
        }
    }
    windows.primary_mut().set_cursor_icon(match state.0 {
        EditorState::Loading => CursorIcon::Wait,
        EditorState::Idle | EditorState::DeletingComponent => {
            if !hovered_comp.is_empty() {
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
}

pub fn world_pos_sy(
    mut texts: Query<&mut Text, With<CursorCoords>>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    let mut text = texts.single_mut();
    text.sections[0].value = format!(
        "x: {} z: {}",
        mouse_pos_world.x.round(),
        mouse_pos_world.y.round()
    );
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

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .with_system(cursor_icon_sy)
                .with_system(crosshair_sy)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .with_system(world_pos_sy)
                .into(),
        )
        .add_exit_system(EditorState::Loading, cursor_setup);
    }
}
