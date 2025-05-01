use std::collections::HashMap;

use bevy::{
    picking::{
        backend::HitData,
        pointer::{Location, PointerAction, PointerInput},
    },
    prelude::*,
};
use itertools::Itertools;

use crate::ui::{cursor::mouse_pos::MousePosWorld, map::window::PointerWithinTilemap};

#[derive(Debug, Clone, Reflect)]
pub struct Click2 {
    pub button: PointerButton,
    pub hit: HitData,
    pub location: Location,
}

#[derive(Debug, Clone, Component)]
pub struct LeftClick(pub Click2);

#[derive(Debug, Clone, Component)]
pub struct MiddleClick(pub Click2);

#[derive(Debug, Clone, Component)]
pub struct RightClick(pub Click2);

#[tracing::instrument(skip_all)]
pub fn on_emit_click2_down(trigger: Trigger<Pointer<Pressed>>, mut commands: Commands) {
    let event = Click2 {
        button: trigger.event.button,
        hit: trigger.event.hit.clone(),
        location: trigger.pointer_location.clone(),
    };
    let mut command = commands.entity(trigger.target());
    match trigger.event.button {
        PointerButton::Primary => command.insert(LeftClick(event)),
        PointerButton::Middle => command.insert(MiddleClick(event)),
        PointerButton::Secondary => command.insert(RightClick(event)),
    };
}

#[tracing::instrument(skip_all)]
pub fn on_emit_click2_up(
    trigger: Trigger<Pointer<Released>>,
    left_click: Query<&LeftClick>,
    middle_click: Query<&MiddleClick>,
    right_click: Query<&RightClick>,
    mut commands: Commands,
    mut event_writer: EventWriter<Pointer<Click2>>,
) {
    let Ok(click_data) = (match trigger.button {
        PointerButton::Primary => left_click.get(trigger.target()).map(|a| {
            commands.entity(trigger.target()).remove::<LeftClick>();
            &a.0
        }),
        PointerButton::Middle => middle_click.get(trigger.target()).map(|a| {
            commands.entity(trigger.target()).remove::<MiddleClick>();
            &a.0
        }),
        PointerButton::Secondary => right_click.get(trigger.target()).map(|a| {
            commands.entity(trigger.target()).remove::<RightClick>();
            &a.0
        }),
    }) else {
        return;
    };
    if trigger.pointer_location != click_data.location {
        return;
    }

    let event = Pointer::new(
        trigger.pointer_id,
        trigger.pointer_location.clone(),
        trigger.target(),
        click_data.to_owned(),
    );
    event_writer.write(event.clone());
    commands.trigger_targets(event, trigger.target());
}

#[tracing::instrument(skip_all)]
pub fn emit_deselect_click_sy(
    mut click_event: ParamSet<(EventReader<Pointer<Click2>>, EventWriter<Pointer<Click2>>)>,
    mut commands: Commands,
    mut input_event: EventReader<PointerInput>,
    pickables: Query<(), With<Pickable>>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
    mouse_pos_world: Res<MousePosWorld>,
    mut old_locations: Local<HashMap<PointerButton, Location>>,
) {
    if pointer_within_tilemap.is_none() {
        return;
    }
    let events = click_event
        .p0()
        .read()
        .filter(|a| a.target != Entity::PLACEHOLDER && pickables.contains(a.target))
        .counts_by(|a| a.button);
    let inputs = input_event.read().collect::<Vec<_>>();
    for button in PointerButton::iter() {
        if let Some(input) = inputs.iter().find(|a| {
            if let PointerAction::Press(b) = a.action {
                b == button
            } else {
                false
            }
        }) {
            old_locations.insert(button, input.location.clone());
        }
        if let Some((input, old_location)) = inputs
            .iter()
            .find(|a| {
                if let PointerAction::Release(b) = a.action {
                    b == button
                } else {
                    false
                }
            })
            .and_then(|input| Some((input, old_locations.remove(&button)?)))
        {
            if events.get(&button).copied().unwrap_or_default() == 0
                && old_location == input.location
            {
                debug!(?button, "Click on no component detected");
                let event = Pointer::new(
                    input.pointer_id,
                    input.location.clone(),
                    Entity::PLACEHOLDER,
                    Click2 {
                        button,
                        location: input.location.clone(),
                        hit: HitData::new(
                            Entity::PLACEHOLDER,
                            0.0,
                            Some(mouse_pos_world.extend(0.0)),
                            None,
                        ),
                    },
                );
                click_event.p1().write(event.clone());
                commands.trigger(event);
            }
        }
    }
}
