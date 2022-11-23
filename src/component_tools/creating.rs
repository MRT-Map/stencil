use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_mouse_tracking_plugin::MousePosWorld;
use bevy_prototype_lyon::entity::ShapeBundle;
use iyes_loopless::prelude::*;
use rand::distributions::{Alphanumeric, DistString};

use crate::{
    component_actions::selecting::{deselect, DeselectQuery},
    cursor::mouse_events::MouseEvent,
    misc::EditorState,
    pla2::{
        bundle::{ComponentBundle, CreatedComponent},
        component::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
    ui::component_panel::PrevNamespaceUsed,
};

const ANGLE_VECTORS: [Vec2; 20] = [
    Vec2::new(4.0, 0.0),
    Vec2::new(4.0, 1.0),
    Vec2::new(3.0, 1.0),
    Vec2::new(2.0, 1.0),
    Vec2::new(1.5, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 1.5),
    Vec2::new(1.0, 2.0),
    Vec2::new(1.0, 3.0),
    Vec2::new(1.0, 4.0),
    Vec2::new(0.0, 4.0),
    Vec2::new(-1.0, 4.0),
    Vec2::new(-1.0, 3.0),
    Vec2::new(-1.0, 2.0),
    Vec2::new(-1.0, 1.5),
    Vec2::new(-1.0, 1.0),
    Vec2::new(-1.5, 1.0),
    Vec2::new(-2.0, 1.0),
    Vec2::new(-3.0, 1.0),
    Vec2::new(-4.0, 1.0),
];

#[tracing::instrument(skip_all)]
pub fn create_point_sy(
    mut commands: Commands,
    mut mouse: EventReader<MouseEvent>,
    skin: Res<Skin>,
    deselect_query: DeselectQuery,
    prev_namespace_used: Res<PrevNamespaceUsed>,
) {
    for event in mouse.iter() {
        if let MouseEvent::LeftClick(_, mouse_pos_world) = event {
            let mut new_point = ComponentBundle::new({
                let mut point = PlaComponent::new(ComponentType::Point);
                point
                    .nodes
                    .push(mouse_pos_world.xy().round().as_ivec2().into());
                point.namespace = prev_namespace_used.0.to_owned();
                point.id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                point
            });
            debug!("Placing new point at {:?}", mouse_pos_world);
            new_point.update_shape(&skin);
            deselect(&mut commands, &deselect_query);
            commands.spawn(new_point);
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn create_component_sy<const IS_AREA: bool>(
    mut set: CreatedQuery,
    mut commands: Commands,
    skin: Res<Skin>,
    mut mouse: EventReader<MouseEvent>,
    mouse_pos_world: Res<MousePosWorld>,
    prev_namespace_used: Res<PrevNamespaceUsed>,
    keys: Res<Input<KeyCode>>,
) {
    let ty = if IS_AREA {
        ComponentType::Area
    } else {
        ComponentType::Line
    };
    if !set.is_empty() {
        let (data, entity) = set.single_mut();
        let mut data = (*data).to_owned();
        let prev_node_pos = data.nodes.last().unwrap().0.as_vec2();
        let mouse_pos_world = mouse_pos_world.xy();
        let next_point =
            if mouse_pos_world != Vec2::ZERO && keys.any_pressed([KeyCode::LAlt, KeyCode::RAlt]) {
                let closest_angle_vec = ANGLE_VECTORS
                    .into_iter()
                    .chain(ANGLE_VECTORS.iter().map(|a| -*a))
                    .min_by_key(|v| {
                        (v.angle_between(mouse_pos_world - prev_node_pos).abs() * 1000.0) as i32
                    })
                    .unwrap();
                (mouse_pos_world - prev_node_pos).project_onto(closest_angle_vec) + prev_node_pos
            } else {
                mouse_pos_world
            };
        data.nodes.push(next_point.round().as_ivec2().into());
        commands.entity(entity).insert(data.get_shape(&skin, false));
    }
    for event in mouse.iter() {
        if let MouseEvent::LeftClick(_, mouse_pos_world) = event {
            let new = mouse_pos_world.xy().round().as_ivec2().into();
            if set.is_empty() {
                let mut new_comp = ComponentBundle::new({
                    let mut point = PlaComponent::new(ty);
                    point.nodes.push(new);
                    point
                });
                debug!("Starting new line/area at {:?}", mouse_pos_world);
                new_comp.update_shape(&skin);
                commands.spawn(new_comp).insert(CreatedComponent);
            } else {
                let (mut data, entity) = set.single_mut();
                if data.nodes.last() == Some(&new) {
                    data.nodes.pop();
                    if data.nodes.is_empty() {
                        commands.entity(entity).despawn_recursive();
                        continue;
                    }
                } else {
                    data.nodes.push(new);
                }
                debug!(
                    ?entity,
                    "Continuing line/area at {:?}",
                    mouse_pos_world.xy().round().as_ivec2()
                );
                commands.entity(entity).insert(data.get_shape(&skin, false));

                if IS_AREA
                    && data.nodes.first() == data.nodes.last()
                    && data.nodes.first().is_some()
                {
                    debug!("Ended on same point, completing area");
                    data.nodes.pop();
                    clear_created_component(&mut commands, &mut set, &skin, &prev_namespace_used.0);
                }
            }
        } else if let MouseEvent::RightClick(_) = event {
            debug!("Completing line/area");
            clear_created_component(&mut commands, &mut set, &skin, &prev_namespace_used.0);
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn clear_created_component(
    commands: &mut Commands,
    created_query: &mut CreatedQuery,
    skin: &Res<Skin>,
    prev_namespace_used: &String,
) {
    for (mut data, entity) in created_query.iter_mut() {
        debug!(?entity, "Clearing CreatedComponent marker");
        if data.nodes.len() == 1 {
            commands.entity(entity).despawn_recursive();
        } else {
            data.namespace = prev_namespace_used.to_owned();
            data.id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
            commands
                .entity(entity)
                .remove::<ShapeBundle>()
                .insert(data.get_shape(skin, false))
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

pub type CreatedQuery<'world, 'state, 'a> =
    Query<'world, 'state, (&'a mut PlaComponent<EditorCoords>, Entity), With<CreatedComponent>>;
