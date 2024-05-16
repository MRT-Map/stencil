use bevy::prelude::*;
use bevy_mouse_tracking::MousePosWorld;
use bevy_prototype_lyon::{prelude::*, shapes::Circle};
use itertools::Itertools;

use crate::{
    component::{
        bundle::{EntityCommandsSelectExt, SelectedComponent},
        pla2::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
    component_actions::undo_redo::{History, UndoRedoAct},
    misc::Action,
    state::EditorState,
    ui::{cursor::mouse_events::MouseEvent, panel::status::Status, UiSet},
};

#[derive(Debug)]
pub struct Orig {
    pub old_pla: PlaComponent<EditorCoords>,
    pub mouse_pos_world: MousePosWorld,
    pub node_pos_world: IVec2,
    pub node_list_pos: usize,
    pub was_new: bool,
}

#[tracing::instrument(skip_all)]
pub fn edit_nodes_sy(
    mut selected: Query<(&mut PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    mut commands: Commands,
    mut orig: Local<Option<Orig>>,
    mut mouse: EventReader<MouseEvent>,
    mouse_pos_world: Res<MousePosWorld>,
    skin: Res<Skin>,
    mut actions: EventWriter<Action>,
    mut status: ResMut<Status>,
) {
    let Ok((mut pla, entity)) = selected.get_single_mut() else {
        return;
    };
    if let Some(orig) = &*orig {
        debug!(?entity, "Moving node");
        pla.nodes[orig.node_list_pos].0 = (mouse_pos_world.xy() - orig.mouse_pos_world.xy()
            + orig.node_pos_world.as_vec2())
        .round()
        .as_ivec2();
        commands.entity(entity).component_display(&skin, &pla);
    }

    let mut clear_orig = false;
    for event in mouse.read() {
        if let MouseEvent::RightPress(mouse_pos_world) = event {
            #[derive(Debug, Eq, PartialEq, Hash)]
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
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            // TODO figure out how to fix this
            let Some((list_pos, world_pos)) = handles.min_by_key(|(_, pos)| {
                mouse_pos_world.xy().distance_squared(pos.as_vec2()) as usize
            }) else {
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
            status.0 = format!("Started movement of node of {}", &*pla).into();
            let (list_pos, was_new) = match list_pos {
                Pos::Existing(i) => (i, false),
                Pos::NewBefore(i) => {
                    pla.nodes.insert(i, EditorCoords(world_pos));
                    (i, true)
                }
            };
            *orig = Some(Orig {
                old_pla: pla.to_owned(),
                mouse_pos_world: *mouse_pos_world,
                node_pos_world: world_pos,
                node_list_pos: list_pos,
                was_new,
            });
        } else if let MouseEvent::RightRelease(_) = event {
            info!(?entity, "Ending movement of node");
            status.0 = format!("Ended movement of node of {}", &*pla).into();
            clear_orig = true;
        } else if let MouseEvent::RightClick(_) = event {
            if let Some(orig) = &*orig {
                if !orig.was_new && pla.get_type(&skin) != Some(ComponentType::Point) {
                    info!(?entity, "Deleting node");
                    status.0 = format!("Deleted node of {}", &*pla).into();
                    pla.nodes.remove(orig.node_list_pos);
                    if pla.nodes.len() < 2 {
                        info!(?entity, "Deleting entity");
                        status.0 = format!("Deleting {}", &*pla).into();
                        commands.entity(entity).despawn_recursive();
                    } else {
                        commands.entity(entity).component_display(&skin, &pla);
                    }
                }
            }
        }
    }
    if clear_orig {
        if let Some(orig) = orig.take() {
            actions.send(Action::new(UndoRedoAct::one_history(History::Component {
                entity,
                before: Some(orig.old_pla.into()),
                after: Some(pla.to_owned().into()),
            })));
        }
    }
}

pub fn update_handles(
    commands: &mut Commands,
    pla: &PlaComponent<EditorCoords>,
    e: Entity,
    skin: &Skin,
    mouse_pos_world: &MousePosWorld,
) {
    trace!("Updating handles");
    commands
        .entity(e)
        .component_display(skin, pla)
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
            (
                ShapeBundle {
                    path: GeometryBuilder::build_as(&Circle {
                        radius: weight * 0.5,
                        center: if pla.get_type(skin) == Some(ComponentType::Point) {
                            Vec2::ZERO
                        } else {
                            coord.as_vec2()
                        },
                    }),
                    spatial: SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 100.0)),
                    ..default()
                },
                Fill::color(Color::WHITE),
                Stroke::new(Color::GRAY, weight * 0.5),
            )
        })
        .map(|bundle| commands.spawn(bundle).id())
        .collect::<Vec<_>>();
    trace!("Pushing first set of children");
    commands.entity(e).push_children(&children);
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
        (
            ShapeBundle {
                path: GeometryBuilder::build_as(&Circle {
                    radius: weight * 0.25,
                    center: coord.as_vec2(),
                }),
                spatial: SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 100.0)),
                ..default()
            },
            Fill::color(Color::WHITE),
            Stroke::new(Color::GRAY, weight * 0.25),
        )
    })
    .map(|bundle| commands.spawn(bundle).id())
    .collect::<Vec<_>>();
    trace!("Pushing second set of children");
    commands.entity(e).push_children(&more_children);
}

#[allow(clippy::needless_pass_by_value)]
pub fn show_handles_sy(
    selected: Query<(&PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    mut commands: Commands,
    skin: Res<Skin>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    for (pla, e) in selected.iter() {
        update_handles(&mut commands, pla, e, &skin, &mouse_pos_world);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn remove_handles_sy(selected: Query<Entity, With<SelectedComponent>>, mut commands: Commands) {
    for e in selected.iter() {
        commands.entity(e).despawn_descendants();
    }
}

pub struct EditNodePlugin;
impl Plugin for EditNodePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            edit_nodes_sy.run_if(in_state(EditorState::EditingNodes)),
        )
        .add_systems(OnExit(EditorState::EditingNodes), remove_handles_sy)
        .add_systems(
            PreUpdate,
            show_handles_sy
                .run_if(in_state(EditorState::EditingNodes))
                .after(UiSet::Reset),
        );
    }
}
