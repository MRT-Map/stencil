use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use rand::distr::{Alphanumeric, SampleString};

use crate::{
    component::{
        actions::selecting::SelectEv,
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
    ui::{cursor::mouse_pos::MousePosWorld, panel::status::Status},
};
use crate::ui::panel::dock::PanelDockState;

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
    trigger: Trigger<Pointer<Click>>,
    pickables: Query<(), With<RayCastPickable>>,
    mut commands: Commands,
    skin: Res<Skin>,
    mut namespaces: ResMut<Namespaces>,
    mut status: ResMut<Status>,
    state: Res<State<EditorState>>,
    panel: Res<PanelDockState>,
) {
    if !panel.pointer_within_tilemap || **state != EditorState::CreatingPoint {
        return;
    }
    if trigger.entity() != Entity::PLACEHOLDER && !pickables.contains(trigger.entity()) {
        return;
    }
    let node = trigger
        .hit
        .position
        .unwrap_or_default()
        .xy()
        .round()
        .as_ivec2();
    let new_point = PointComponentBundle::new(
        {
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
        },
        &skin,
    );
    debug!("Placing new point at {node:?}");
    status.0 = format!("Created new point {} at {:?}", new_point.data, node).into();

    commands.trigger(SelectEv::DeselectAll);

    let pla = new_point.data.clone();
    let entity = commands.spawn(new_point).id();
    commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
        entity,
        before: None,
        after: Some(pla.into()),
    }));
}

#[tracing::instrument(skip_all)]
pub fn on_line_area_left_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    state: Res<State<EditorState>>,
    mut set: CreatedQuery,
    mut status: ResMut<Status>,
    skin: Res<Skin>,
    panel: Res<PanelDockState>,
) {
    if !panel.pointer_within_tilemap || trigger.button != PointerButton::Primary {
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
    if let Ok((entity, mut data)) = set.get_single_mut() {
        if data.nodes.last().map(|a| a.0) == Some(new) {
            data.nodes.pop();
            if data.nodes.is_empty() {
                commands.entity(entity).despawn_recursive();
                return;
            }
        } else {
            data.nodes.push(new.into());
        }
        debug!(?entity, "Continuing {ty_text} at {new:?}");
        status.0 = format!("Continuing {ty_text} at {new:?}").into();
        commands.entity(entity).component_display(&skin, &data);

        if ty_text == "area" && data.nodes.first() == data.nodes.last() && !data.nodes.is_empty() {
            debug!("Ended on same point, completing area");
            data.nodes.pop();
            commands.trigger(ClearCreatedComponentEv);
        }
    } else {
        commands.trigger(SelectEv::DeselectAll);

        let data = {
            let mut point = PlaComponent::new(ty);
            point.nodes.push(new.into());
            point
        };
        debug!("Starting new {ty_text} at {new:?}");
        status.0 = format!("Starting new {ty_text} at {new:?}",).into();
        if ty_text == "area" {
            commands.spawn(AreaComponentBundle::new(data, &skin))
        } else {
            commands.spawn(LineComponentBundle::new(data, &skin))
        }
        .insert(CreatedComponent);
    }
}

#[tracing::instrument(skip_all)]
pub fn on_line_area_right_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    state: Res<State<EditorState>>,
    panel: Res<PanelDockState>,
) {
    if !panel.pointer_within_tilemap || trigger.button != PointerButton::Secondary {
        return;
    }
    if ![EditorState::CreatingArea, EditorState::CreatingLine].contains(&state) {
        return;
    }

    debug!("Completing line/area");
    commands.trigger(ClearCreatedComponentEv);
}

#[tracing::instrument(skip_all)]
pub fn create_component_sy(
    set: CreatedQuery,
    skin: Res<Skin>,
    mouse_pos_world: Res<MousePosWorld>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let Ok((entity, data)) = set.get_single() else {
        return;
    };
    let mut data = (*data).clone();
    let prev_node_pos = data.nodes.last().unwrap().0.as_vec2();
    let next_point = if **mouse_pos_world != Vec2::ZERO
        && keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
    {
        #[expect(clippy::cast_possible_truncation)] // TODO find some way to fix this
        let closest_angle_vec = ANGLE_VECTORS
            .into_iter()
            .chain(ANGLE_VECTORS.iter().map(|a| -*a))
            .min_by_key(|v| (v.angle_to(**mouse_pos_world - prev_node_pos).abs() * 1000.0) as i32)
            .unwrap();
        (**mouse_pos_world - prev_node_pos).project_onto(closest_angle_vec) + prev_node_pos
    } else {
        **mouse_pos_world
    };
    data.nodes.push(next_point.round().as_ivec2().into());
    commands.entity(entity).component_display(&skin, &data);
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
    let Ok((entity, mut data)) = created_query.get_single_mut() else {
        return;
    };
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
        data.id = Alphanumeric.sample_string(&mut rand::rng(), 16);
        commands
            .entity(entity)
            .remove::<ShapeBundle>()
            .component_display(&skin, &data)
            .remove::<CreatedComponent>();
        commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
            entity,
            before: None,
            after: Some(data.to_owned().into()),
        }));
        status.0 = format!(
            "Created new {} {}",
            if data.get_type(&skin) == ComponentType::Area {
                "area"
            } else {
                "line"
            },
            &*data
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
    Query<'world, 'state, (Entity, &'a mut PlaComponent<EditorCoords>), With<CreatedComponent>>;

#[derive(Copy, Clone, Event)]
pub struct ClearCreatedComponentEv;
