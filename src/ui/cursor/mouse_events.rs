use std::time::Duration;

use bevy::{
    picking::{
        backend::HitData,
        pointer::{PointerAction, PointerInput, PressDirection},
    },
    prelude::*,
};
use bevy_egui::EguiContexts;
use itertools::Itertools;

use crate::ui::{
    cursor::mouse_pos::MousePosWorld,
    panel::dock::{within_tilemap, PanelDockState},
};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HoveredComponent;

#[tracing::instrument(skip_all)]
pub fn click_handler_sy(
    mut pointer_event: ParamSet<(EventReader<Pointer<Click>>, EventWriter<Pointer<Click>>)>,
    mut commands: Commands,
    mut input_event: EventReader<PointerInput>,
    mut ctx: EguiContexts,
    panel: Res<PanelDockState>,
    mouse_pos_world: Res<MousePosWorld>,
) {
    if !within_tilemap(&mut ctx, &panel) {
        return;
    }
    let events = pointer_event.p0().read().counts_by(|a| a.button);
    let inputs = input_event.read().collect::<Vec<_>>();
    for button in PointerButton::iter() {
        if events.get(&button).copied().unwrap_or_default() == 0 {
            if let Some(input) = inputs.iter().find(|a| {
                matches!(
                    a.action,
                    PointerAction::Pressed {
                        direction: PressDirection::Up,
                        button
                    }
                )
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
