use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_mouse_tracking_plugin::MousePosWorld;
use bevy_prototype_lyon::entity::ShapeBundle;
use iyes_loopless::prelude::*;

use crate::{
    editor::{
        bundles::component::{ComponentBundle, CreatedComponent},
        component_actions::selecting::deselect,
        cursor::mouse_events::MouseEvent,
    },
    types::{
        pla::{EditorCoords, PlaComponent},
        skin::Skin,
        ComponentType, CreatedQuery, DeselectQuery, EditorState,
    },
};

#[tracing::instrument(skip_all)]
pub fn create_point_sy(
    mut commands: Commands,
    mut events: EventReader<MouseEvent>,
    skin: Res<Skin>,
    deselect_query: DeselectQuery,
) {
    for event in events.iter() {
        if let MouseEvent::LeftClick(_, mouse_pos_world) = event {
            let mut new_point = ComponentBundle::new({
                let mut point = PlaComponent::new(ComponentType::Point);
                point
                    .nodes
                    .push(mouse_pos_world.xy().round().as_ivec2().into());
                point
            });
            debug!("Placing new point at {:?}", mouse_pos_world);
            new_point.update_shape(&skin);
            deselect(&mut commands, &deselect_query);
            commands.spawn_bundle(new_point);
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn create_component_sy<const IS_AREA: bool>(
    mut set: CreatedQuery,
    mut commands: Commands,
    skin: Res<Skin>,
    mut events: EventReader<MouseEvent>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    let ty = if IS_AREA {
        ComponentType::Area
    } else {
        ComponentType::Line
    };
    if !set.is_empty() {
        let (data, entity): (Mut<PlaComponent<EditorCoords>>, Entity) = set.single_mut();
        let mut data = (*data).to_owned();
        data.nodes
            .push(mouse_pos_world.xy().round().as_ivec2().into());
        commands
            .entity(entity)
            .insert_bundle(data.get_shape(&skin, false));
    }
    for event in events.iter() {
        if let MouseEvent::LeftClick(_, mouse_pos_world) = event {
            if set.is_empty() {
                let mut new_comp = ComponentBundle::new({
                    let mut point = PlaComponent::new(ty);
                    point
                        .nodes
                        .push(mouse_pos_world.xy().round().as_ivec2().into());
                    point
                });
                debug!("Starting new line/area at {:?}", mouse_pos_world);
                new_comp.update_shape(&skin);
                commands.spawn_bundle(new_comp).insert(CreatedComponent);
            } else {
                let (mut data, entity): (Mut<PlaComponent<EditorCoords>>, Entity) =
                    set.single_mut();
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

                if IS_AREA
                    && data.nodes.first() == data.nodes.last()
                    && data.nodes.first().is_some()
                {
                    debug!("Ended on same point, completing area");
                    data.nodes.pop();
                    clear_created_component(&mut commands, &set, &skin);
                }
            }
        } else if let MouseEvent::RightClick(_) = event {
            debug!("Completing line/area");
            clear_created_component(&mut commands, &set, &skin);
        }
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
            commands.entity(entity).despawn_recursive();
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
                .run_in_state(EditorState::CreatingComponent(ComponentType::Line))
                .with_system(create_component_sy::<false>)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(EditorState::CreatingComponent(ComponentType::Area))
                .with_system(create_component_sy::<true>)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(EditorState::CreatingComponent(ComponentType::Point))
                .with_system(create_point_sy)
                .into(),
        );
    }
}
