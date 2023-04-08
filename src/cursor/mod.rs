use bevy::{math::Vec3Swizzles, prelude::*, sprite::Anchor};
use bevy_mouse_tracking_plugin::MousePosWorld;

use crate::{
    cursor::mouse_events::HoveredComponent,
    misc::{CustomSet, EditorState},
    tilemap::{settings::TileSettings, zoom::Zoom},
    ui::HoveringOverGui,
};

pub mod mouse_events;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Crosshair;

#[tracing::instrument(skip_all)]
pub fn crosshair_sy(
    mut commands: Commands,
    state: Option<Res<State<EditorState>>>,
    mut ch: Query<(Entity, &mut Transform, &mut Sprite), With<Crosshair>>,
    server: Res<AssetServer>,
    zoom: Res<Zoom>,
    mouse_pos_world: Res<MousePosWorld>,
    tile_settings: Res<TileSettings>,
) {
    if let Some(state) = state {
        if state.0.component_type().is_none() {
            for (e, _, _) in ch.iter() {
                debug!("Despawning crosshair");
                commands.entity(e).despawn_recursive();
            }
            return;
        }
    } else {
        return;
    }
    let translation = mouse_pos_world.round().xy();
    let new_transform = Transform::from_translation(translation.extend(100.0));
    let new_custom_size = Some(Vec2::splat(
        (f32::from(tile_settings.max_tile_zoom) - zoom.0).exp2() * 16f32,
    ));
    if ch.is_empty() {
        debug!("Spawning crosshair");
        commands
            .spawn(SpriteBundle {
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
    mut windows: Query<&mut Window>,
    state: Option<Res<State<EditorState>>>,
    hovering_over_gui: Res<HoveringOverGui>,
    hovered_comp: Query<(), With<HoveredComponent>>,
) {
    let state = if let Some(state) = state {
        state.0
    } else {
        EditorState::Loading
    };
    for mut window in windows.iter_mut() {
        if state.component_type().is_some() {
            window.cursor.visible = hovering_over_gui.0;
            continue;
        }
        window.cursor.visible = true;
        if hovering_over_gui.0 {
            continue;
        }
        window.cursor.icon = (match state {
            EditorState::Loading => CursorIcon::Wait,
            EditorState::Idle | EditorState::DeletingComponent | EditorState::EditingNodes => {
                if !hovered_comp.is_empty() {
                    CursorIcon::Hand
                } else if buttons.pressed(MouseButton::Left) {
                    CursorIcon::Grabbing
                } else {
                    CursorIcon::Grab
                }
            }
            EditorState::CreatingLine | EditorState::CreatingArea | EditorState::CreatingPoint => {
                unreachable!()
            }
        });
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(CustomSet::Cursor.before(CoreSet::Update))
            .add_systems((cursor_icon_sy, crosshair_sy).in_base_set(CustomSet::Cursor))
            .add_plugin(mouse_events::MouseEventsPlugin);
    }
}
