use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use crate::{
    component::pla2::{EditorCoords, PlaComponent},
    history::{HistoryEntry, HistoryEv},
    ui::panel::status::Status,
};
use crate::component::actions::rendering::RenderEv;
use crate::component::actions::selecting::SelectedComponent;
use crate::component::pla2::ComponentType;
use crate::component::skin::Skin;
use crate::state::EditorState;
use crate::ui::cursor::mouse_pos::MousePosWorld;
use crate::ui::panel::dock::PanelDockState;

#[derive(Debug, Clone, Component)]
pub struct MoveData {
    pub old_mouse_pos_world: MousePosWorld,
    pub old_translation: Vec3,
}

#[tracing::instrument(skip_all)]
pub fn on_right_click_drag(
    trigger: Trigger<Pointer<Drag>>,
    mut query: Query<(&mut Transform, &MoveData), With<SelectedComponent>>,
    panel: Res<PanelDockState>,
    mouse_pos_world: Res<MousePosWorld>,
    state: Res<State<EditorState>>,
) {
    if !panel.pointer_within_tilemap || trigger.button != PointerButton::Secondary || *state != EditorState::Idle {
        return;
    }
    let Ok((mut transform, move_data)) = query.get_mut(trigger.entity()) else {
        return;
    };

    transform.translation.x = (move_data.old_translation.x + mouse_pos_world.x - move_data.old_mouse_pos_world.x).round();
    transform.translation.y = (move_data.old_translation.x + mouse_pos_world.y - move_data.old_mouse_pos_world.y).round();
}

#[tracing::instrument(skip_all)]
pub fn on_right_click_drag_start(
    trigger: Trigger<Pointer<DragStart>>,
    mut commands: Commands,
    query: Query<(&PlaComponent<EditorCoords>, &Transform), With<SelectedComponent>>,
    mut status: ResMut<Status>,
    panel: Res<PanelDockState>,
    mouse_pos_world: Res<MousePosWorld>,
    state: Res<State<EditorState>>,
) {
    if !panel.pointer_within_tilemap || trigger.button != PointerButton::Secondary || *state != EditorState::Idle {
        return;
    }
    let e = trigger.entity();
    let Ok((pla, transform)) = query.get(e) else {
        return;
    };
    commands.entity(e).insert(MoveData {
        old_mouse_pos_world: *mouse_pos_world,
        old_translation: transform.translation,
    });
    info!("Started move");
    status.0 = format!("Started moving {pla}").into();
}

#[tracing::instrument(skip_all)]
pub fn on_right_click_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut PlaComponent<EditorCoords>, &MoveData), With<SelectedComponent>>,
    mut status: ResMut<Status>,
    panel: Res<PanelDockState>,
    skin: Res<Skin>,
    mouse_pos_world: Res<MousePosWorld>,
    state: Res<State<EditorState>>,
) {
    if !panel.pointer_within_tilemap || trigger.button != PointerButton::Secondary || *state != EditorState::Idle {
        return;
    }
    let e = trigger.entity();
    let Ok((mut transform, mut pla, move_data)) = query.get_mut(e) else {
        return;
    };
    if pla.get_type(&skin) != ComponentType::Point {
        transform.translation.x = 0.0; // transform.translation.x.round();
        transform.translation.y = 0.0; // transform.translation.y.round();
    }

    let old_pla = pla.to_owned();
    for node in &mut pla.nodes {
        node.0 += (**mouse_pos_world - *move_data.old_mouse_pos_world).round().as_ivec2();
    }
    commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
        e,
        before: Some(old_pla.into()),
        after: Some(pla.to_owned().into()),
    }));
    commands.entity(e).remove::<(Aabb, MoveData)>().trigger(RenderEv::default());
    status.0 = format!("Moved component {}", &*pla).into();
    info!("Ended move");
}

pub struct MoveComponentPlugin;
impl Plugin for MoveComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_right_click_drag)
            .add_observer(on_right_click_drag_start)
            .add_observer(on_right_click_drag_end);
    }
}
