use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    component::{
        actions::{rendering::RenderEv, selecting::SelectedComponent},
        pla2::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
    history::{HistoryEntry, HistoryEv},
    misc_config::settings::MiscSettings,
    state::EditorState,
    ui::{
        cursor::{mouse_events::Click2, mouse_pos::MousePosWorld},
        map::{window::PointerWithinTilemap, zoom::Zoom},
        panel::status::Status,
    },
};

#[derive(Debug, Clone, Component)]
pub struct NodeEditData {
    pub old_pla: PlaComponent,
    pub mouse_pos_world: MousePosWorld,
    pub node_pos_world: IVec2,
    pub node_list_pos: usize,
    pub was_new: bool,
}

#[tracing::instrument(skip_all)]
pub fn on_node_edit_right_down(
    trigger: Trigger<Pointer<Pressed>>,
    mut selected: Query<(Entity, &mut PlaComponent), With<SelectedComponent>>,
    mut commands: Commands,
    mouse_pos_world: Res<MousePosWorld>,
    skin: Res<Skin>,
    mut status: ResMut<Status>,
    zoom: Res<Zoom>,
    misc_settings: Res<MiscSettings>,
    state: Res<State<EditorState>>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none()
        || trigger.button != PointerButton::Secondary
        || **state != EditorState::EditingNodes
    {
        return;
    }
    let Ok((e, mut pla)) = selected.single_mut() else {
        return;
    };

    #[derive(Debug, Eq, PartialEq, Hash)]
    #[expect(clippy::items_after_statements)]
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
            if pla.get_skin_type(&skin) == ComponentType::Area {
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
        warn!(?e, "Component has no points");
        return;
    };
    if mouse_pos_world.distance_squared(world_pos.as_vec2())
        > (2048.0 / zoom.0.exp2() * misc_settings.big_handle_size).powi(2)
    {
        info!(?e, "Handle is too far");
        return;
    }
    info!(?e, ?list_pos, "Starting movement of node");
    status.0 = format!("Started movement of node of {}", &*pla).into();
    let (list_pos, was_new) = match list_pos {
        Pos::Existing(i) => (i, false),
        Pos::NewBefore(i) => {
            pla.nodes.insert(i, EditorCoords(world_pos));
            (i, true)
        }
    };
    commands.entity(e).insert(NodeEditData {
        old_pla: pla.to_owned(),
        mouse_pos_world: *mouse_pos_world,
        node_pos_world: world_pos,
        node_list_pos: list_pos,
        was_new,
    });
}

#[tracing::instrument(skip_all)]
pub fn on_node_edit_right_up(
    trigger: Trigger<Pointer<Released>>,
    mut selected: Query<(Entity, &mut PlaComponent), (With<SelectedComponent>, With<NodeEditData>)>,
    mut commands: Commands,
    mut status: ResMut<Status>,
    state: Res<State<EditorState>>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none()
        || trigger.button != PointerButton::Secondary
        || **state != EditorState::EditingNodes
    {
        return;
    }
    let Ok((e, pla)) = selected.single_mut() else {
        return;
    };

    info!(?e, "Ending movement of node");
    status.0 = format!("Ended movement of node of {}", &*pla).into();

    commands.trigger_targets(EditNodesEv::ClearEventData, e);
}

#[tracing::instrument(skip_all)]
pub fn on_node_edit_right_click(
    trigger: Trigger<Pointer<Click2>>,
    mut selected: Query<(Entity, &mut PlaComponent, &NodeEditData), With<SelectedComponent>>,
    mut commands: Commands,
    skin: Res<Skin>,
    mut status: ResMut<Status>,
    state: Res<State<EditorState>>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none()
        || trigger.button != PointerButton::Secondary
        || **state != EditorState::EditingNodes
    {
        return;
    }
    let Ok((e, mut pla, orig)) = selected.single_mut() else {
        return;
    };

    if orig.was_new || pla.get_skin_type(&skin) == ComponentType::Point {
        return;
    }
    info!(?e, "Deleting node");
    status.0 = format!("Deleted node of {}", &*pla).into();
    pla.nodes.remove(orig.node_list_pos);
    if pla.nodes.len() < 2 {
        info!(?e, "Deleting entity");
        status.0 = format!("Deleting {}", &*pla).into();
        commands.entity(e).despawn();
    } else {
        commands.entity(e).trigger(RenderEv::default());
    }
}
#[tracing::instrument(skip_all)]
pub fn on_edit_nodes(
    trigger: Trigger<EditNodesEv>,
    mut selected: Query<(Entity, &PlaComponent, &NodeEditData), With<SelectedComponent>>,
    mut commands: Commands,
) {
    let Ok((e, pla, orig)) = selected.single_mut() else {
        return;
    };
    match trigger.event() {
        EditNodesEv::ClearEventData => {
            commands
                .entity(e)
                .remove::<NodeEditData>()
                .trigger(RenderEv::default());
            commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
                e,
                before: Some(orig.to_owned().old_pla.into()),
                after: Some(pla.to_owned().into()),
            }));
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn move_selected_node_sy(
    mut selected: Query<(Entity, &mut PlaComponent, &NodeEditData), With<SelectedComponent>>,
    mut commands: Commands,
    mouse_pos_world: Res<MousePosWorld>,
) {
    let Ok((e, mut pla, orig)) = selected.single_mut() else {
        return;
    };

    debug!(?e, "Moving node");
    pla.nodes[orig.node_list_pos].0 = (**mouse_pos_world - *orig.mouse_pos_world
        + orig.node_pos_world.as_vec2())
    .round()
    .as_ivec2();
    commands.entity(e).trigger(RenderEv::default());
}

pub struct EditNodePlugin;
impl Plugin for EditNodePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            move_selected_node_sy.run_if(in_state(EditorState::EditingNodes)),
        )
        .add_observer(on_node_edit_right_down)
        .add_observer(on_node_edit_right_up)
        .add_observer(on_node_edit_right_click)
        .add_observer(on_edit_nodes);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Event)]
pub enum EditNodesEv {
    ClearEventData,
}
