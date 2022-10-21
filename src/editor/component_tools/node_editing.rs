use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_mouse_tracking_plugin::MousePosWorld;
use itertools::Itertools;
use iyes_loopless::condition::ConditionSet;

use crate::{
    editor::{bundles::component::SelectedComponent, cursor::mouse_events::MouseEvent},
    types::{
        pla::{EditorCoords, PlaComponent},
        skin::Skin,
        ComponentType, EditorState,
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
    mut events: EventReader<MouseEvent>,
    mouse_pos_world: Res<MousePosWorld>,
    skin: Res<Skin>,
) {
    let (mut pla, entity): (Mut<PlaComponent<EditorCoords>>, Entity) =
        if let Ok(query_data) = selected.get_single_mut() {
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
        commands
            .entity(entity)
            .insert_bundle(pla.get_shape(&skin, true));
    }

    let mut clear_orig = false;
    for event in events.iter() {
        if let MouseEvent::RightPress(mouse_pos_world) = event {
            #[derive(Debug, Eq, PartialEq, Hash)]
            #[allow(dead_code)]
            enum Pos {
                Existing(usize),
                NewBefore(usize),
            }
            let handles =
                pla.nodes
                    .iter()
                    .enumerate()
                    .map(|(i, ec)| (Pos::Existing(i), ec.0))
                    .chain(
                        pla.nodes.iter().enumerate().tuple_windows::<(_, _)>().map(
                            |((_, this), (i, next))| (Pos::NewBefore(i), (this.0 + next.0) / 2),
                        ),
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
                        commands
                            .entity(entity)
                            .insert_bundle(pla.get_shape(&skin, true));
                    }
                }
            }
        }
    }
    if clear_orig {
        *orig = None;
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
        );
    }
}
