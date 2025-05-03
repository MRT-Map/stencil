use bevy::prelude::*;
use rand::distr::{Alphanumeric, SampleString};

use crate::{
    component::{
        actions::{rendering::RenderEv, selecting::SelectEv},
        make_component,
        pla2::{ComponentType, PlaComponent},
        skin::Skin,
    },
    history::{HistoryEntry, HistoryEv},
    project::Namespaces,
    state::EditorState,
    ui::{
        cursor::{mouse_events::Click2, mouse_pos::MousePosWorld},
        map::window::PointerWithinTilemap,
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
pub fn on_point_left_click(
    trigger: Trigger<Pointer<Click2>>,
    pickables: Query<(), With<Pickable>>,
    mut commands: Commands,
    skin: Res<Skin>,
    mut namespaces: ResMut<Namespaces>,
    mut status: ResMut<Status>,
    state: Res<State<EditorState>>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none()
        || **state != EditorState::CreatingPoint
        || trigger.target() != Entity::PLACEHOLDER && !pickables.contains(trigger.target())
    {
        return;
    }

    let node = trigger
        .hit
        .position
        .unwrap_or_default()
        .xy()
        .round()
        .as_ivec2();
    let pla = {
        let mut point = PlaComponent::new(ComponentType::Point);
        point.nodes.push(node.into());
        if !namespaces
            .visibilities
            .get(&namespaces.prev_used)
            .copied()
            .unwrap_or_default()
        {
            namespaces.prev_used = "_misc".into();
        }
        namespaces.prev_used.clone_into(&mut point.namespace);
        point.id = Alphanumeric.sample_string(&mut rand::rng(), 16);
        point
    };
    let new_point = make_component(pla.clone(), &skin);
    debug!("Placing new point at {node:?}");
    status.0 = format!("Created new point {pla} at {node:?}").into();

    commands.trigger(SelectEv::DeselectAll);

    let e = commands.spawn(new_point).id();
    commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
        e,
        before: None,
        after: Some(pla.into()),
    }));
}

#[tracing::instrument(skip_all)]
pub fn on_line_area_left_click(
    trigger: Trigger<Pointer<Click2>>,
    pickables: Query<(), With<Pickable>>,
    mut commands: Commands,
    state: Res<State<EditorState>>,
    mut set: CreatedQuery,
    mut status: ResMut<Status>,
    skin: Res<Skin>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none()
        || trigger.button != PointerButton::Primary
        || trigger.target() != Entity::PLACEHOLDER && !pickables.contains(trigger.target())
    {
        return;
    }

    let (ty, ty_text) = match **state {
        EditorState::CreatingArea => (ComponentType::Area, "area"),
        EditorState::CreatingLine => (ComponentType::Line, "line"),
        _ => {
            return;
        }
    };

    let new = trigger
        .hit
        .position
        .unwrap_or_default()
        .xy()
        .round()
        .as_ivec2();
    if let Ok((e, mut pla)) = set.single_mut() {
        if pla.nodes.last().map(|a| a.0) == Some(new) {
            pla.nodes.pop();
            if pla.nodes.is_empty() {
                commands.entity(e).despawn();
                return;
            }
        } else {
            pla.nodes.push(new.into());
        }
        debug!(?e, "Continuing {ty_text} at {new:?}");
        status.0 = format!("Continuing {ty_text} at {new:?}").into();
        commands.entity(e).trigger(RenderEv::default());

        if ty_text == "area" && pla.nodes.first() == pla.nodes.last() && !pla.nodes.is_empty() {
            debug!("Ended on same point, completing area");
            pla.nodes.pop();
            commands.trigger(ClearCreatedComponentEv);
        }
    } else {
        commands.trigger(SelectEv::DeselectAll);

        let pla = {
            let mut pla = PlaComponent::new(ty);
            pla.id = Alphanumeric.sample_string(&mut rand::rng(), 16);
            pla.nodes.push(new.into());
            pla
        };
        debug!("Starting new {ty_text} at {new:?}");
        status.0 = format!("Starting new {ty_text} at {new:?}",).into();
        commands
            .spawn(make_component(pla, &skin))
            .insert(CreatedComponent);
    }
}

#[tracing::instrument(skip_all)]
pub fn on_line_area_right_click(
    trigger: Trigger<Pointer<Click2>>,
    pickables: Query<(), With<Pickable>>,
    mut commands: Commands,
    state: Res<State<EditorState>>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none()
        || trigger.button != PointerButton::Secondary
        || trigger.target() != Entity::PLACEHOLDER && !pickables.contains(trigger.target())
        || ![EditorState::CreatingArea, EditorState::CreatingLine].contains(&state)
    {
        return;
    }

    debug!("Completing line/area");
    commands.trigger(ClearCreatedComponentEv);
}

#[tracing::instrument(skip_all)]
pub fn create_component_sy(
    set: Query<(Entity, &PlaComponent), With<CreatedComponent>>,
    mouse_pos_world: Res<MousePosWorld>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let Ok((e, pla)) = set.single() else {
        return;
    };
    let mut pla = (*pla).clone();
    let prev_node_pos = pla.nodes.last().unwrap().0.as_vec2();
    let next_point = if **mouse_pos_world != Vec2::ZERO
        && keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
    {
        let closest_angle_vec = ANGLE_VECTORS
            .into_iter()
            .chain(ANGLE_VECTORS.iter().map(|a| -*a))
            .min_by_key(|v| (v.angle_to(**mouse_pos_world - prev_node_pos).abs() * 1000.0) as i32)
            .unwrap();
        (**mouse_pos_world - prev_node_pos).project_onto(closest_angle_vec) + prev_node_pos
    } else {
        **mouse_pos_world
    };
    pla.nodes.push(next_point.round().as_ivec2().into());
    commands.entity(e).trigger(RenderEv(Some(pla)));
}

#[tracing::instrument(skip_all)]
pub fn on_clear_created_component(
    _trigger: Trigger<ClearCreatedComponentEv>,
    mut commands: Commands,
    mut created_query: CreatedQuery,
    skin: Res<Skin>,
    mut namespaces: ResMut<Namespaces>,
    mut status: ResMut<Status>,
) {
    let Ok((e, mut pla)) = created_query.single_mut() else {
        return;
    };
    debug!(?e, "Clearing CreatedComponent marker");
    if pla.nodes.len() == 1 {
        commands.entity(e).despawn();
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
        namespaces.prev_used.clone_into(&mut pla.namespace);
        commands
            .entity(e)
            .trigger(RenderEv::default())
            .remove::<CreatedComponent>();
        commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
            e,
            before: None,
            after: Some(pla.to_owned().into()),
        }));
        status.0 = format!(
            "Created new {} {}",
            if pla.get_skin_type(&skin) == ComponentType::Area {
                "area"
            } else {
                "line"
            },
            &*pla
        )
        .into();
    }
}

pub struct CreateComponentPlugin;
impl Plugin for CreateComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            create_component_sy.run_if(
                in_state(EditorState::CreatingLine).or(in_state(EditorState::CreatingArea)),
            ),
        )
        .add_observer(on_point_left_click)
        .add_observer(on_line_area_left_click)
        .add_observer(on_line_area_right_click)
        .add_observer(on_clear_created_component);
    }
}

pub type CreatedQuery<'world, 'state, 'a> =
    Query<'world, 'state, (Entity, &'a mut PlaComponent), With<CreatedComponent>>;

#[derive(Copy, Clone, Event)]
pub struct ClearCreatedComponentEv;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct CreatedComponent;
