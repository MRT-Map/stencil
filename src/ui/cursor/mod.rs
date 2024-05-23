use bevy::{prelude::*, sprite::Anchor};
use bevy_egui::{egui, EguiContexts};
use bevy_mouse_tracking::MousePosWorld;

use crate::{
    init::load_assets::ImageAssets,
    misc_config::settings::MiscSettings,
    state::{EditorState, IntoSystemConfigExt},
    tile::zoom::Zoom,
    ui::{
        cursor::mouse_events::{HoveredComponent, MouseEvent},
        panel::dock::{within_tilemap, PanelDockState},
        tilemap::settings::TileSettings,
        UiSchedule, UiSet,
    },
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
    images: Res<ImageAssets>,
    zoom: Res<Zoom>,
    mouse_pos_world: Res<MousePosWorld>,
    mut ctx: EguiContexts,
    tile_settings: Res<TileSettings>,
    panel: Res<PanelDockState>,
    misc_settings: Res<MiscSettings>,
) {
    if let Some(state) = state {
        if state.component_type().is_none()
            || (state.component_type().is_some() && !within_tilemap(&mut ctx, &panel))
        {
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
        (f32::from(tile_settings.basemaps[0].max_tile_zoom) - zoom.0).exp2()
            * 16f32
            * misc_settings.crosshair_size,
    ));
    if ch.is_empty() {
        debug!("Spawning crosshair");
        commands
            .spawn(SpriteBundle {
                texture: images.crosshair.to_owned(),
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
    buttons: Res<ButtonInput<MouseButton>>,
    mut windows: Query<(Entity, &mut Window)>,
    mut ctx: EguiContexts,
    state: Option<Res<State<EditorState>>>,
    hovered_comp: Query<(), With<HoveredComponent>>,
    panel: Res<PanelDockState>,
) {
    let state = if let Some(state) = state {
        **state
    } else {
        EditorState::Loading
    };

    for (e, mut window) in &mut windows {
        if state.component_type().is_some() {
            window.cursor.visible = !within_tilemap(&mut ctx, &panel);
            continue;
        }
        window.cursor.visible = true;
        if !within_tilemap(&mut ctx, &panel) {
            continue;
        }

        ctx.ctx_for_window_mut(e).set_cursor_icon(match state {
            EditorState::Loading => egui::CursorIcon::Wait,
            EditorState::Idle | EditorState::DeletingComponent | EditorState::EditingNodes => {
                if !hovered_comp.is_empty() {
                    egui::CursorIcon::PointingHand
                } else if buttons.pressed(MouseButton::Left) {
                    egui::CursorIcon::Grabbing
                } else {
                    egui::CursorIcon::Grab
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
        app.add_systems(
            UiSchedule,
            (
                mouse_events::left_click_handler_sy,
                mouse_events::right_click_handler_sy,
                mouse_events::hover_handler_sy,
            )
                .in_set(UiSet::Mouse),
        )
        .add_systems(
            PostUpdate,
            (cursor_icon_sy, crosshair_sy.run_if_not_loading()),
        )
        .add_event::<MouseEvent>();
    }
}
