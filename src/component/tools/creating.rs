use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use rand::distributions::{Alphanumeric, DistString};

use crate::{
    component::{
        actions::selecting::{deselect, DeselectQuery},
        bundle::{
            AreaComponentBundle, CreatedComponent, EntityCommandsSelectExt, LineComponentBundle,
            PointComponentBundle,
        },
        pla2::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
    history::{HistoryEntry, HistoryEv},
    project::Namespaces,
    state::EditorState,
    ui::{
        cursor::{mouse_events::MouseEvent, mouse_pos::MousePosWorld},
        panel::status::Status,
    },
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
    mut namespaces: ResMut<Namespaces>,
    mut status: ResMut<Status>,
) {
    for event in mouse.read() {
        if let MouseEvent::LeftClick(_, mouse_pos_world) = event {
            let new_point = PointComponentBundle::new(
                {
                    let mut point = PlaComponent::new(ComponentType::Point);
                    point.nodes.push(mouse_pos_world.round().as_ivec2().into());
                    if !namespaces
                        .visibilities
                        .get(&namespaces.prev_used)
                        .copied()
                        .unwrap_or_default()
                    {
                        namespaces.prev_used = "_misc".into();
                    }
                    namespaces.prev_used.clone_into(&mut point.namespace);
                    point.id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                    point
                },
                &skin,
            );
            debug!("Placing new point at {mouse_pos_world:?}");
            status.0 = format!(
                "Created new point {} at {:?}",
                new_point.data,
                mouse_pos_world.round().as_ivec2()
            )
            .into();
            deselect(&mut commands, &deselect_query);
            let pla = new_point.data.clone();
            let entity = commands.spawn(new_point).id();
            commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
                entity,
                before: None,
                after: Some(pla.into()),
            }));
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn create_component_sy<const IS_AREA: bool>(
    mut set: CreatedQuery,
    skin: Res<Skin>,
    mut mouse: EventReader<MouseEvent>,
    mouse_pos_world: Res<MousePosWorld>,
    mut namespaces: ResMut<Namespaces>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut status: ResMut<Status>,
) {
    let ty = if IS_AREA {
        ComponentType::Area
    } else {
        ComponentType::Line
    };
    let ty_text = if IS_AREA { "area" } else { "line" };
    if let Ok((data, entity)) = set.get_single_mut() {
        let mut data = (*data).clone();
        let prev_node_pos = data.nodes.last().unwrap().0.as_vec2();
        let next_point = if **mouse_pos_world != Vec2::ZERO
            && keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
        {
            #[expect(clippy::cast_possible_truncation)] // TODO find some way to fix this
            let closest_angle_vec = ANGLE_VECTORS
                .into_iter()
                .chain(ANGLE_VECTORS.iter().map(|a| -*a))
                .min_by_key(|v| {
                    (v.angle_between(**mouse_pos_world - prev_node_pos).abs() * 1000.0) as i32
                })
                .unwrap();
            (**mouse_pos_world - prev_node_pos).project_onto(closest_angle_vec) + prev_node_pos
        } else {
            **mouse_pos_world
        };
        data.nodes.push(next_point.round().as_ivec2().into());
        commands.entity(entity).component_display(&skin, &data);
    }
    for event in mouse.read() {
        if let MouseEvent::LeftClick(_, mouse_pos_world) = event {
            let new = mouse_pos_world.round().as_ivec2().into();
            if set.is_empty() {
                let data = {
                    let mut point = PlaComponent::new(ty);
                    point.nodes.push(new);
                    point
                };
                debug!("Starting new {ty_text} at {mouse_pos_world:?}");
                status.0 = format!(
                    "Starting new {ty_text} at {:?}",
                    mouse_pos_world.round().as_ivec2()
                )
                .into();
                if IS_AREA {
                    commands.spawn(AreaComponentBundle::new(data, &skin))
                } else {
                    commands.spawn(LineComponentBundle::new(data, &skin))
                }
                .insert(CreatedComponent);
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
                    "Continuing {ty_text} at {:?}",
                    mouse_pos_world.round().as_ivec2()
                );
                status.0 = format!(
                    "Continuing {ty_text} at {:?}",
                    mouse_pos_world.round().as_ivec2()
                )
                .into();
                commands.entity(entity).component_display(&skin, &data);

                if IS_AREA
                    && data.nodes.first() == data.nodes.last()
                    && data.nodes.first().is_some()
                {
                    debug!("Ended on same point, completing area");
                    data.nodes.pop();
                    clear_created_component(
                        &mut commands,
                        &mut set,
                        &skin,
                        &mut namespaces,
                        &mut status,
                        ty_text,
                    );
                }
            }
        } else if let MouseEvent::RightClick(_) = event {
            debug!("Completing line/area");
            clear_created_component(
                &mut commands,
                &mut set,
                &skin,
                &mut namespaces,
                &mut status,
                ty_text,
            );
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn clear_created_component(
    commands: &mut Commands,
    created_query: &mut CreatedQuery,
    skin: &Res<Skin>,
    namespaces: &mut ResMut<Namespaces>,
    status: &mut ResMut<Status>,
    ty_text: &str,
) {
    for (mut data, entity) in &mut *created_query {
        debug!(?entity, "Clearing CreatedComponent marker");
        if data.nodes.len() == 1 {
            commands.entity(entity).despawn_recursive();
            status.0 = "Cancelled component creation".into();
        } else {
            if !namespaces
                .visibilities
                .get(&namespaces.prev_used)
                .copied()
                .unwrap_or_default()
            {
                namespaces.prev_used = "_misc".into();
            }
            namespaces.prev_used.clone_into(&mut data.namespace);
            data.id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
            commands
                .entity(entity)
                .remove::<ShapeBundle>()
                .component_display(skin, &data)
                .remove::<CreatedComponent>();
            commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
                entity,
                before: None,
                after: Some(data.to_owned().into()),
            }));
            status.0 = format!("Created new {ty_text} {}", &*data).into();
        }
    }
}

pub struct CreateComponentPlugin;
impl Plugin for CreateComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            create_component_sy::<false>.run_if(in_state(EditorState::CreatingLine)), //.before(on_state_change),
        )
        .add_systems(
            Update,
            create_component_sy::<true>.run_if(in_state(EditorState::CreatingArea)), //.before(on_state_change),
        )
        .add_systems(
            Update,
            create_point_sy.run_if(in_state(EditorState::CreatingPoint)),
        );
    }
}

pub type CreatedQuery<'world, 'state, 'a> =
    Query<'world, 'state, (&'a mut PlaComponent<EditorCoords>, Entity), With<CreatedComponent>>;
