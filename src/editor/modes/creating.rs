use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_mouse_tracking_plugin::MousePosWorld;
use bevy_prototype_lyon::entity::ShapeBundle;
use iyes_loopless::prelude::*;

use crate::{
    editor::{
        bundles::component::{ComponentBundle, CreatedComponent},
        modes::selecting::deselect,
        ui::HoveringOverGui,
    },
    types::{
        pla::{EditorCoords, PlaComponent},
        skin::Skin,
        ComponentType, CreatedQuery, DeselectQuery, DetectMouseMoveOnClick,
        DetectMouseMoveOnClickExt, EditorState,
    },
};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn create_component_sy(
    mut set: ParamSet<(CreatedQuery, DeselectQuery)>,
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    skin: Res<Skin>,
    state: Res<CurrentState<EditorState>>,
    hovering_over_gui: Res<HoveringOverGui>,
    mut mm_detector: DetectMouseMoveOnClick,
    mouse_pos_world: Res<MousePosWorld>,
) {
    let ty = if let EditorState::CreatingComponent(ty) = &state.0 {
        ty
    } else {
        return;
    };
    mm_detector.handle_press(&buttons);
    if buttons.just_released(MouseButton::Left) && !hovering_over_gui.0 {
        if mm_detector.handle_release() {
            debug!("Mouse move detected, won't place new point");
            return;
        };
        if *ty == ComponentType::Point {
            let mut new_point = ComponentBundle::new({
                let mut point = PlaComponent::new(ty.to_owned());
                point
                    .nodes
                    .push(mouse_pos_world.xy().round().as_ivec2().into());
                point
            });
            debug!("Placing new point at {:?}", mouse_pos_world);
            new_point.update_shape(&skin);
            deselect(&mut commands, &set.p1());
            commands.spawn_bundle(new_point);
            return;
        }
        if set.p0().is_empty() {
            let mut new_comp = ComponentBundle::new({
                let mut point = PlaComponent::new(ty.to_owned());
                point
                    .nodes
                    .push(mouse_pos_world.xy().round().as_ivec2().into());
                point
            });
            debug!("Starting new line/area at {:?}", mouse_pos_world);
            new_comp.update_shape(&skin);
            commands.spawn_bundle(new_comp).insert(CreatedComponent);
        } else {
            let mut created_query = set.p0();
            let (mut data, entity): (Mut<PlaComponent<EditorCoords>>, Entity) =
                created_query.single_mut();
            match data.get_type(&skin).unwrap() {
                ComponentType::Line | ComponentType::Area => {
                    data.nodes
                        .push(mouse_pos_world.xy().round().as_ivec2().into());
                    debug!(
                        ?entity,
                        "Continuing line/area at {:?}",
                        mouse_pos_world.xy().round().as_ivec2()
                    );
                    commands
                        .entity(entity)
                        .insert_bundle(data.get_shape(&skin, false));

                    if data.get_type(&skin).unwrap() == ComponentType::Area
                        && data.nodes.first() == data.nodes.last()
                        && data.nodes.first().is_some()
                    {
                        debug!("Ended on same point, completing area");
                        clear_created_component(&mut commands, &set.p0(), &skin);
                    }
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
        clear_created_component(&mut commands, &set.p0(), &skin);
    } else if *ty != ComponentType::Point && !set.p0().is_empty() {
        let mut created_query = set.p0();
        let (data, entity): (Mut<PlaComponent<EditorCoords>>, Entity) = created_query.single_mut();
        let mut data = (*data).to_owned();
        data.nodes
            .push(mouse_pos_world.xy().round().as_ivec2().into());
        commands
            .entity(entity)
            .insert_bundle(data.get_shape(&skin, false));
    }
}

#[tracing::instrument(skip_all)]
pub fn clear_created_component(
    commands: &mut Commands,
    created_query: &CreatedQuery,
    skin: &Res<Skin>,
) {
    for (data, entity) in created_query.iter() {
        debug!(?entity, "Clearing CreatedComponent marker");
        if data.nodes.len() == 1 {
            commands.entity(entity).despawn();
        } else {
            commands
                .entity(entity)
                .remove_bundle::<ShapeBundle>()
                .insert_bundle(data.get_shape(skin, false))
                .remove::<CreatedComponent>();
        }
    }
}

pub struct CreateComponentPlugin;
impl Plugin for CreateComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .with_system(create_component_sy)
                .into(),
        );
    }
}
