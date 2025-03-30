use bevy::{color::palettes::basic::GRAY, prelude::*};
use itertools::Itertools;

use crate::{
    component::{
        actions::selecting::highlight_selected_sy,
        bundle::{EntityCommandsSelectExt, SelectedComponent},
        circle::circle,
        pla2::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
    history::{HistoryEntry, HistoryEv},
    misc_config::settings::MiscSettings,
    state::EditorState,
    tile::zoom::Zoom,
    ui::{cursor::mouse_pos::MousePosWorld, panel::status::Status, UiSet},
};

#[derive(Debug, Clone, Component)]
pub struct NodeEditData {
    pub old_pla: PlaComponent<EditorCoords>,
    pub mouse_pos_world: MousePosWorld,
    pub node_pos_world: IVec2,
    pub node_list_pos: usize,
    pub was_new: bool,
}

#[tracing::instrument(skip_all)]
pub fn on_node_edit_right_down(
    trigger: Trigger<Pointer<Down>>,
    mut selected: Query<(Entity, &mut PlaComponent<EditorCoords>), With<SelectedComponent>>,
    mut commands: Commands,
    mouse_pos_world: Res<MousePosWorld>,
    skin: Res<Skin>,
    mut status: ResMut<Status>,
    zoom: Res<Zoom>,
    misc_settings: Res<MiscSettings>,
) {
    if trigger.button != PointerButton::Secondary {
        return;
    }
    if trigger.entity() == Entity::PLACEHOLDER {
        return;
    }
    let Ok((entity, mut pla)) = selected.get_single_mut() else {
        return;
    };

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
            if pla.get_type(&skin) == ComponentType::Area {
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
    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    // TODO figure out how to fix this
    let Some((list_pos, world_pos)) =
        handles.min_by_key(|(_, pos)| mouse_pos_world.distance_squared(pos.as_vec2()) as usize)
    else {
        warn!(?entity, "Component has no points");
        return;
    };
    if mouse_pos_world.distance_squared(world_pos.as_vec2())
        > (2048.0 / zoom.0.exp2() * misc_settings.big_handle_size).powi(2)
    {
        info!(?entity, "Handle is too far");
        return;
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
    commands.entity(entity).insert(NodeEditData {
        old_pla: pla.to_owned(),
        mouse_pos_world: *mouse_pos_world,
        node_pos_world: world_pos,
        node_list_pos: list_pos,
        was_new,
    });
}

#[tracing::instrument(skip_all)]
pub fn on_node_edit_right_up(
    trigger: Trigger<Pointer<Up>>,
    mut selected: Query<
        (Entity, &mut PlaComponent<EditorCoords>),
        (With<SelectedComponent>, With<NodeEditData>),
    >,
    mut commands: Commands,
    mut status: ResMut<Status>,
) {
    if trigger.button != PointerButton::Secondary {
        return;
    }
    if trigger.entity() == Entity::PLACEHOLDER {
        return;
    }
    let Ok((entity, mut pla)) = selected.get_single_mut() else {
        return;
    };

    info!(?entity, "Ending movement of node");
    status.0 = format!("Ended movement of node of {}", &*pla).into();

    commands.trigger_targets(EditNodesEv::ClearEventData, entity);
}

#[tracing::instrument(skip_all)]
pub fn on_node_edit_right_click(
    trigger: Trigger<Pointer<Click>>,
    mut selected: Query<
        (Entity, &mut PlaComponent<EditorCoords>, &NodeEditData),
        With<SelectedComponent>,
    >,
    mut commands: Commands,
    skin: Res<Skin>,
    mut status: ResMut<Status>,
) {
    if trigger.button != PointerButton::Secondary {
        return;
    }
    if trigger.entity() == Entity::PLACEHOLDER {
        return;
    }
    let Ok((entity, mut pla, orig)) = selected.get_single_mut() else {
        return;
    };

    if orig.was_new || pla.get_type(&skin) == ComponentType::Point {
        return;
    }
    info!(?entity, "Deleting node");
    status.0 = format!("Deleted node of {}", &*pla).into();
    pla.nodes.remove(orig.node_list_pos);
    if pla.nodes.len() < 2 {
        info!(?entity, "Deleting entity");
        status.0 = format!("Deleting {}", &*pla).into();
        commands.entity(entity).despawn_recursive();
    } else {
        commands.entity(entity).select_component(&skin, &pla);
    }
}
#[tracing::instrument(skip_all)]
pub fn on_edit_nodes(
    trigger: Trigger<EditNodesEv>,
    mut selected: Query<
        (Entity, &PlaComponent<EditorCoords>, &NodeEditData),
        With<SelectedComponent>,
    >,
    mut commands: Commands,
) {
    let Ok((entity, pla, orig)) = selected.get_single_mut() else {
        return;
    };
    match trigger.event() {
        EditNodesEv::ClearEventData => {
            commands.entity(entity).remove::<NodeEditData>();
            commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
                entity,
                before: Some(orig.to_owned().old_pla.into()),
                after: Some(pla.to_owned().into()),
            }));
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn move_selected_node_sy(
    mut selected: Query<
        (Entity, &mut PlaComponent<EditorCoords>, &NodeEditData),
        With<SelectedComponent>,
    >,
    mut commands: Commands,
    mouse_pos_world: Res<MousePosWorld>,
    skin: Res<Skin>,
) {
    let Ok((entity, mut pla, orig)) = selected.get_single_mut() else {
        return;
    };

    debug!(?entity, "Moving node");
    pla.nodes[orig.node_list_pos].0 = (**mouse_pos_world - *orig.mouse_pos_world
        + orig.node_pos_world.as_vec2())
    .round()
    .as_ivec2();
    commands.entity(entity).select_component(&skin, &pla);
}

pub fn update_handles(
    commands: &mut Commands,
    pla: &PlaComponent<EditorCoords>,
    e: Entity,
    skin: &Skin,
    mouse_pos_world: &MousePosWorld,
    zoom: &Zoom,
    misc_settings: &MiscSettings,
) {
    trace!("Updating handles");
    commands
        .entity(e)
        .select_component(skin, pla)
        .despawn_descendants();
    let children = pla
        .nodes
        .iter()
        .map(|coord| &coord.0)
        .filter(|coord| {
            if pla.nodes.len() > misc_settings.hide_far_handles_threshold {
                (coord.as_vec2() - **mouse_pos_world).length_squared()
                    < misc_settings.hide_far_handles_distance
            } else {
                true
            }
        })
        .map(|coord| {
            circle(
                zoom,
                if pla.get_type(skin) == ComponentType::Point {
                    Vec2::ZERO
                } else {
                    coord.as_vec2()
                },
                misc_settings.big_handle_size,
                GRAY.into(),
            )
        })
        .map(|bundle| commands.spawn(bundle).id())
        .collect::<Vec<_>>();
    trace!("Pushing first set of children");
    commands.entity(e).add_children(&children);
    let more_children = if pla.get_type(skin) == ComponentType::Area {
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
        if pla.nodes.len() > misc_settings.hide_far_handles_threshold {
            (coord.as_vec2() - **mouse_pos_world).length_squared()
                < misc_settings.hide_far_handles_distance
        } else {
            true
        }
    })
    .map(|coord| {
        circle(
            zoom,
            coord.as_vec2(),
            misc_settings.small_handle_size,
            GRAY.into(),
        )
    })
    .map(|bundle| commands.spawn(bundle).id())
    .collect::<Vec<_>>();
    trace!("Pushing second set of children");
    commands.entity(e).add_children(&more_children);
}

#[expect(clippy::needless_pass_by_value)]
pub fn show_handles_sy(
    selected: Query<(&PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    mut commands: Commands,
    skin: Res<Skin>,
    mouse_pos_world: Res<MousePosWorld>,
    zoom: Res<Zoom>,
    misc_settings: Res<MiscSettings>,
) {
    for (pla, e) in selected.iter() {
        update_handles(
            &mut commands,
            pla,
            e,
            &skin,
            &mouse_pos_world,
            &zoom,
            &misc_settings,
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
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
            move_selected_node_sy.run_if(in_state(EditorState::EditingNodes)),
        )
        .add_systems(OnExit(EditorState::EditingNodes), remove_handles_sy)
        .add_systems(
            PreUpdate,
            show_handles_sy
                .run_if(in_state(EditorState::EditingNodes))
                .after(UiSet::Reset)
                .after(highlight_selected_sy),
        );
    }
}

#[derive(Copy, Clone, Event)]
pub enum EditNodesEv {
    ClearEventData,
}
