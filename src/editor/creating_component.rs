use bevy::prelude::*;
use bevy_mouse_tracking_plugin::MainCamera;
use bevy_prototype_lyon::entity::ShapeBundle;
use iyes_loopless::prelude::*;

use crate::{
    editor::{
        bundles::component::{ComponentBundle, CreatedComponent, EditorComponent},
        cursor::get_cursor_world_pos,
        selecting_component::{deselect, select_query},
        ui::HoveringOverGui,
    },
    types::{
        ComponentType, CreatedQuery, DeselectQuery, DetectMouseMoveOnClick, DetectMouseMoveOnClickExt,
        EditorState, pla::ComponentCoords, SelectQuery, skin::Skin,
    },
};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn create_component(
    mut set: ParamSet<(
        CreatedQuery,
        DeselectQuery,
        SelectQuery<With<CreatedComponent>>,
    )>,
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    skin: Res<Skin>,
    state: Res<CurrentState<EditorState>>,
    windows: Res<Windows>,
    mut camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    hovering_over_gui: Res<HoveringOverGui>,
    mut mm_detector: DetectMouseMoveOnClick,
) {
    if let EditorState::CreatingComponent(type_) = &state.0 {
        let (camera, transform): (&Camera, &GlobalTransform) = camera.single_mut();
        let mouse_world_pos = if let Some(mp) = get_cursor_world_pos(&windows, camera, transform) {
            mp
        } else {
            return;
        };
        if buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right) {
            mm_detector.handle_press(&buttons);
        }
        if buttons.just_released(MouseButton::Left) && !hovering_over_gui.0 {
            if mm_detector.handle_release() {
                debug!("Mouse move detected, won't place new point");
                return;
            };
            if *type_ == ComponentType::Point {
                let mut new_point = ComponentBundle::new(
                    EditorComponent::new(type_.to_owned()),
                    mouse_world_pos.as_ivec2(),
                );
                debug!("Placing new point at {:?}", mouse_world_pos);
                new_point.update_shape(&skin);
                deselect(&mut commands, &set.p1());
                commands.spawn_bundle(new_point);
                return;
            }
            if set.p0().is_empty() {
                let mut new_comp = ComponentBundle::new(
                    EditorComponent::new(type_.to_owned()),
                    mouse_world_pos.as_ivec2(),
                );
                debug!("Starting new line/area at {:?}", mouse_world_pos);
                new_comp.update_shape(&skin);
                commands.spawn_bundle(new_comp).insert(CreatedComponent);
            } else {
                let mut created_query = set.p0();
                let (data, mut coords, entity): (&EditorComponent, Mut<ComponentCoords>, Entity) =
                    created_query.single_mut();
                match data.get_type(&skin).unwrap() {
                    ComponentType::Line | ComponentType::Area => {
                        coords.0.push(mouse_world_pos.as_ivec2());
                        debug!(
                            ?entity,
                            "Continuing line/area at {}, {}",
                            mouse_world_pos.as_ivec2().x,
                            mouse_world_pos.as_ivec2().y
                        );
                        commands.entity(entity).insert_bundle(data.get_shape(
                            (*coords).to_owned(),
                            &skin,
                            false,
                        ));
                    }
                    ComponentType::Point => unreachable!(),
                }
            }
        } else if buttons.just_released(MouseButton::Right) && !hovering_over_gui.0 {
            if mm_detector.handle_release() {
                debug!("Mouse move detected, won't complete line/area");
                return;
            };
            debug!("Completing line/area");
            select_query(&mut commands, &mut set.p2());
        } else if *type_ != ComponentType::Point && !set.p0().is_empty() {
            let mut created_query = set.p0();
            let (data, coords, entity): (&EditorComponent, Mut<ComponentCoords>, Entity) =
                created_query.single_mut();
            let mut coords = coords.to_owned();
            coords.0.push(mouse_world_pos.as_ivec2());
            commands
                .entity(entity)
                .insert_bundle(data.get_shape(coords, &skin, false));
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn clear_created_component(
    commands: &mut Commands,
    created_query: &CreatedQuery,
    skin: &Res<Skin>,
) {
    for (data, coords, entity) in created_query.iter() {
        debug!(?entity, "Clearing CreatedComponent marker");
        commands
            .entity(entity)
            .remove_bundle::<ShapeBundle>()
            .insert_bundle(data.get_shape(coords.to_owned(), skin, false))
            .remove::<CreatedComponent>();
    }
}

pub struct CreateComponentPlugin;
impl Plugin for CreateComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .with_system(create_component)
                .into(),
        );
    }
}
