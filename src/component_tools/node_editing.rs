use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_mouse_tracking_plugin::MousePosWorld;
use bevy_prototype_lyon::{prelude::*, shapes::Circle};
use itertools::Itertools;
use iyes_loopless::{condition::ConditionSet, prelude::AppLooplessStateExt};

use crate::{
    cursor::mouse_events::MouseEvent,
    misc::{CustomStage, EditorState},
    pla2::{
        bundle::SelectedComponent,
        component::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
};

#[derive(Debug)]
pub struct Orig {
    pub mouse_pos_world: MousePosWorld,
    pub node_pos_world: IVec2,
    pub node_list_pos: usize,
    pub was_new: bool,
}

#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn edit_nodes_sy(
    mut selected: Query<(&mut PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    mut commands: Commands,
    mut orig: Local<Option<Orig>>,
    mut mouse: EventReader<MouseEvent>,
    mouse_pos_world: Res<MousePosWorld>,
    skin: Res<Skin>,
) {
    let (mut pla, entity) = if let Ok(query_data) = selected.get_single_mut() {
        query_data
    } else {
        return;
    };
    if let Some(orig) = &*orig {
        debug!(?entity, "Moving node");
        pla.nodes[orig.node_list_pos].0 = (mouse_pos_world.xy() - orig.mouse_pos_world.xy()
            + orig.node_pos_world.as_vec2())
        .round()
        .as_ivec2();
        commands.entity(entity).insert(pla.get_shape(&skin, true));
    }

    let mut clear_orig = false;
    for event in mouse.iter() {
        if let MouseEvent::RightPress(mouse_pos_world) = event {
            #[derive(Debug, Eq, PartialEq, Hash)]
            #[allow(dead_code)]
            enum Pos {
                Existing(usize),
                NewBefore(usize),
            }
            let handles = pla
                .nodes
                .iter()
                .enumerate()
                .map(|(i, ec)| (Pos::Existing(i), ec.0))
                .chain(
                    if pla.get_type(&skin) == Some(ComponentType::Area) {
                        pla.nodes
                            .iter()
                            .enumerate()
                            .circular_tuple_windows::<(_, _)>()
                            .collect::<Vec<_>>()
                            .into_iter()
                    } else {
                        pla.nodes
                            .iter()
                            .enumerate()
                            .tuple_windows::<(_, _)>()
                            .collect::<Vec<_>>()
                            .into_iter()
                    }
                    .map(|((_, this), (i, next))| (Pos::NewBefore(i), (this.0 + next.0) / 2)),
                );
            let (list_pos, world_pos) = if let Some(h) = handles.min_by_key(|(_, pos)| {
                mouse_pos_world.xy().distance_squared(pos.as_vec2()) as usize
            }) {
                h
            } else {
                warn!(?entity, "Component has no points");
                continue;
            };
            if mouse_pos_world.xy().distance_squared(world_pos.as_vec2())
                > pla.weight(&skin).unwrap_or(2) as f32
            {
                info!(?entity, "Handle is too far");
                continue;
            }
            info!(?entity, ?list_pos, "Starting movement of node");
            let (list_pos, was_new) = match list_pos {
                Pos::Existing(i) => (i, false),
                Pos::NewBefore(i) => {
                    pla.nodes.insert(i, EditorCoords(world_pos));
                    (i, true)
                }
            };
            *orig = Some(Orig {
                mouse_pos_world: *mouse_pos_world,
                node_pos_world: world_pos,
                node_list_pos: list_pos,
                was_new,
            })
        } else if let MouseEvent::RightRelease(_) = event {
            info!(?entity, "Ending movement of node");
            clear_orig = true
        } else if let MouseEvent::RightClick(_) = event {
            if let Some(orig) = &*orig {
                if !orig.was_new && pla.get_type(&skin) != Some(ComponentType::Point) {
                    info!(?entity, "Deleting node");
                    pla.nodes.remove(orig.node_list_pos);
                    if pla.nodes.len() < 2 {
                        info!(?entity, "Deleting entity");
                        commands.entity(entity).despawn_recursive()
                    } else {
                        commands.entity(entity).insert(pla.get_shape(&skin, true));
                    }
                }
            }
        }
    }
    if clear_orig {
        *orig = None;
    }
}

pub fn update_handles(
    commands: &mut Commands,
    pla: &PlaComponent<EditorCoords>,
    e: &Entity,
    skin: &Skin,
    mouse_pos_world: &MousePosWorld,
) {
    trace!("Updating handles");
    commands
        .entity(*e)
        .insert(pla.get_shape(skin, true))
        .despawn_descendants();
    let children = pla
        .nodes
        .iter()
        .map(|coord| &coord.0)
        .filter(|coord| {
            if pla.nodes.len() > 50 {
                (coord.as_vec2() - mouse_pos_world.xy()).length_squared() < 10000.0
            } else {
                true
            }
        })
        .map(|coord| {
            let weight = pla.weight(skin).unwrap_or(2) as f32;
            GeometryBuilder::build_as(
                &Circle {
                    radius: weight * 0.5,
                    center: if pla.get_type(skin) == Some(ComponentType::Point) {
                        Vec2::ZERO
                    } else {
                        coord.as_vec2()
                    },
                },
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::WHITE),
                    outline_mode: StrokeMode::new(Color::GRAY, weight * 0.5),
                },
                Transform::from_xyz(0.0, 0.0, 100.0),
            )
        })
        .map(|bundle| commands.spawn(bundle).id())
        .collect::<Vec<_>>();
    trace!("Pushing first set of children");
    commands.entity(*e).push_children(&children);
    let more_children = if pla.get_type(skin) == Some(ComponentType::Area) {
        pla.nodes
            .iter()
            .circular_tuple_windows::<(_, _)>()
            .collect::<Vec<_>>()
            .into_iter()
    } else {
        pla.nodes
            .iter()
            .tuple_windows::<(_, _)>()
            .collect::<Vec<_>>()
            .into_iter()
    }
    .map(|(c1, c2)| (c1.0 + c2.0) / 2)
    .filter(|coord| {
        if pla.nodes.len() > 50 {
            (coord.as_vec2() - mouse_pos_world.xy()).length_squared() < 10000.0
        } else {
            true
        }
    })
    .map(|coord| {
        let weight = pla.weight(skin).unwrap_or(2) as f32;
        GeometryBuilder::build_as(
            &Circle {
                radius: weight * 0.25,
                center: coord.as_vec2(),
            },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::GRAY, weight * 0.25),
            },
            Transform::from_xyz(0.0, 0.0, 100.0),
        )
    })
    .map(|bundle| commands.spawn(bundle).id())
    .collect::<Vec<_>>();
    trace!("Pushing second set of children");
    commands.entity(*e).push_children(&more_children);
}

pub fn show_handles_sy(
    selected: Query<(&PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    mut commands: Commands,
    skin: Res<Skin>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    for (pla, e) in selected.iter() {
        update_handles(&mut commands, pla, &e, &skin, &mouse_pos_world)
    }
}

pub fn remove_handles_sy(selected: Query<Entity, With<SelectedComponent>>, mut commands: Commands) {
    for e in selected.iter() {
        commands.entity(e).despawn_descendants();
    }
}

pub struct EditNodePlugin;
impl Plugin for EditNodePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(EditorState::EditingNodes)
                .with_system(edit_nodes_sy)
                .into(),
        )
        .add_exit_system(EditorState::EditingNodes, remove_handles_sy)
        .add_system_set_to_stage(
            CustomStage::Ui,
            ConditionSet::new()
                .run_in_state(EditorState::EditingNodes)
                .with_system(show_handles_sy)
                .into(),
        );
    }
}
