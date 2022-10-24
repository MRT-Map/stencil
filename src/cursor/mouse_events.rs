use bevy::prelude::*;
use bevy_mod_picking::{HoverEvent, PickingEvent};
use bevy_mouse_tracking_plugin::{MousePos, MousePosWorld};
use iyes_loopless::condition::ConditionSet;

use crate::{
    misc::EditorState,
    ui::{HoveringOverGui, UiStage},
};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HoveredComponent;

pub const CLICK_MAX_OFFSET: f32 = 25.0;

#[derive(Debug)]
pub enum MouseEvent {
    HoverOver(Entity),
    HoverLeave(Entity),
    LeftPress(Option<Entity>, MousePosWorld),
    LeftRelease(Option<Entity>, MousePosWorld),
    LeftClick(Option<Entity>, MousePosWorld),
    RightPress(MousePosWorld),
    RightRelease(MousePosWorld),
    RightClick(MousePosWorld),
}

#[tracing::instrument(skip_all)]
pub fn right_click_handler_sy(
    mut event_writer: EventWriter<MouseEvent>,
    hovering_over_gui: Res<HoveringOverGui>,
    buttons: Res<Input<MouseButton>>,
    mut prev_mouse_pos: Local<Option<MousePos>>,
    mouse_pos: Res<MousePos>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    if buttons.just_pressed(MouseButton::Right) && !hovering_over_gui.0 {
        debug!("RightPress detected");
        *prev_mouse_pos = Some(*mouse_pos);
        event_writer.send(MouseEvent::RightPress(*mouse_pos_world));
    }
    if buttons.just_released(MouseButton::Right) && !hovering_over_gui.0 {
        debug!("RightRelease detected");
        event_writer.send(MouseEvent::RightRelease(*mouse_pos_world));
        if let Some(prev) = *prev_mouse_pos {
            if (*prev - **mouse_pos).length_squared() <= CLICK_MAX_OFFSET && !hovering_over_gui.0 {
                debug!("RightClick detected");
                event_writer.send(MouseEvent::RightClick(*mouse_pos_world))
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all)]
pub fn left_click_handler_sy(
    mut commands: Commands,
    mut event_reader: EventReader<PickingEvent>,
    mut event_writer: EventWriter<MouseEvent>,
    hovering_over_gui: Res<HoveringOverGui>,
    buttons: Res<Input<MouseButton>>,
    mut selected_entity: Local<Option<Entity>>,
    mut prev_mouse_pos: Local<Option<MousePos>>,
    mouse_pos: Res<MousePos>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    let mut pressed_on_comp = false;
    for event in event_reader.iter() {
        if let PickingEvent::Clicked(e) = event {
            if !hovering_over_gui.0 {
                debug!(?e, "Press detected");
                *selected_entity = Some(*e);
                *prev_mouse_pos = Some(*mouse_pos);
                event_writer.send(MouseEvent::LeftPress(Some(*e), *mouse_pos_world));
                pressed_on_comp = true;
            }
        } else if let PickingEvent::Hover(ev) = event {
            match ev {
                HoverEvent::JustEntered(e) => {
                    trace!(?e, "HoverOver detected");
                    event_writer.send(MouseEvent::HoverOver(*e));
                    commands.entity(*e).insert(HoveredComponent);
                }
                HoverEvent::JustLeft(e) => {
                    trace!(?e, "HoverLeave detected");
                    event_writer.send(MouseEvent::HoverLeave(*e));
                    commands.entity(*e).remove::<HoveredComponent>();
                }
            };
        }
    }
    if buttons.just_pressed(MouseButton::Left) && !pressed_on_comp {
        debug!(e = ?Option::<Entity>::None, "LeftPress detected");
        *prev_mouse_pos = Some(*mouse_pos);
        *selected_entity = None;
        event_writer.send(MouseEvent::LeftPress(None, *mouse_pos_world));
    }
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let prev = if let Some(prev) = *prev_mouse_pos {
        *prev_mouse_pos = None;
        prev
    } else {
        return;
    };
    let curr = *mouse_pos;
    debug!(e = ?selected_entity, "LeftRelease detected");
    event_writer.send(MouseEvent::LeftRelease(*selected_entity, *mouse_pos_world));
    if (*prev - *curr).length_squared() <= CLICK_MAX_OFFSET && !hovering_over_gui.0 {
        debug!(e = ?selected_entity, "LeftClick detected");
        event_writer.send(MouseEvent::LeftClick(*selected_entity, *mouse_pos_world))
    }
}

pub struct MouseEventsPlugin;
impl Plugin for MouseEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MouseEvent>()
            .add_stage_after(UiStage, "cursor_events", SystemStage::parallel())
            .add_system_set_to_stage(
                "cursor_events",
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .with_system(left_click_handler_sy)
                    .with_system(right_click_handler_sy)
                    .into(),
            );
    }
}
