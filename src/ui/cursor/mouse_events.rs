use std::time::Duration;

use bevy::{
    picking::{
        backend::HitData,
        pointer::{PointerAction, PointerInput, PressDirection},
    },
    prelude::*,
};
use itertools::Itertools;

use crate::ui::{
    cursor::mouse_pos::MousePosWorld,
    panel::dock::{PanelDockState},
};

#[tracing::instrument(skip_all)]
pub fn click_handler_sy(
    mut pointer_event: ParamSet<(EventReader<Pointer<Click>>, EventWriter<Pointer<Click>>)>,
    mut commands: Commands,
    mut input_event: EventReader<PointerInput>,
    pickables: Query<(), With<RayCastPickable>>,
    panel: Res<PanelDockState>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    if !panel.pointer_within_tilemap {
        return;
    }
    let events = pointer_event
        .p0()
        .read()
        .filter(|a| a.target != Entity::PLACEHOLDER && pickables.contains(a.target))
        .counts_by(|a| a.button);
    let inputs = input_event.read().collect::<Vec<_>>();
    for button in PointerButton::iter() {
        if events.get(&button).copied().unwrap_or_default() == 0 {
            if let Some(input) = inputs.iter().find(|a| {
                if let PointerAction::Pressed {
                    direction: PressDirection::Up,
                    button: b,
                } = a.action
                {
                    b == button
                } else {
                    false
                }
            }) {
                debug!(?button, "Click on no component detected");
                let event = Pointer::new(
                    Entity::PLACEHOLDER,
                    input.pointer_id,
                    input.location.clone(),
                    Click {
                        button,
                        hit: HitData::new(
                            Entity::PLACEHOLDER,
                            0.0,
                            Some(mouse_pos_world.extend(0.0)),
                            None,
                        ),
                        duration: Duration::default(),
                    },
                );
                pointer_event.p1().send(event.clone());
                commands.trigger(event);
            }
        }
    }
}
