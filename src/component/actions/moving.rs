use bevy::prelude::*;

use crate::{
    component::{
        bundle::SelectedComponent,
        pla2::{EditorCoords, PlaComponent},
    },
    history::{HistoryEntry, HistoryEv},
    state::{EditorState, IntoSystemConfigExt},
    ui::{
        cursor::{mouse_events::HoveredComponent, mouse_pos::MousePosWorld},
        panel::status::Status,
    },
};

#[tracing::instrument(skip_all)]
pub fn on_right_click_drag(
    trigger: Trigger<Pointer<Drag>>,
    mut query: Query<(&mut Transform), With<SelectedComponent>>,
) {
    if trigger.button != PointerButton::Secondary {
        return;
    }
    let Ok(mut transform) = query.get_mut(trigger.entity()) else {
        return;
    };

    transform.translation.x += trigger.event.delta.x;
    transform.translation.y += trigger.event.delta.y;
}

#[tracing::instrument(skip_all)]
pub fn on_right_click_drag_start(
    trigger: Trigger<Pointer<DragStart>>,
    query: Query<&PlaComponent<EditorCoords>, With<SelectedComponent>>,
    mut status: ResMut<Status>,
) {
    if trigger.button != PointerButton::Secondary {
        return;
    }
    let Ok(pla) = query.get(trigger.entity()) else {
        return;
    };
    info!("Started move");
    status.0 = format!("Started moving {}", &*pla).into();
}

#[tracing::instrument(skip_all)]
pub fn on_right_click_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut PlaComponent<EditorCoords>), With<SelectedComponent>>,
    mut status: ResMut<Status>,
) {
    if trigger.button != PointerButton::Secondary {
        return;
    }
    let Ok((mut transform, mut pla)) = query.get_mut(trigger.entity()) else {
        return;
    };
    transform.translation.x = transform.translation.x.round();
    transform.translation.y = transform.translation.y.round();

    let old_pla = pla.to_owned();
    for node in &mut pla.nodes {
        node.0 += trigger.distance.round().as_ivec2();
    }
    commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
        entity: trigger.entity(),
        before: Some(old_pla.into()),
        after: Some(pla.to_owned().into()),
    }));
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
