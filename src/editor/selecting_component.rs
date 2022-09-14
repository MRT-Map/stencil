use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_mod_picking::PickingEvent;
use iyes_loopless::prelude::*;

use crate::{
    editor::shadow::{SelectShadow, SelectShadowBundle},
    pla::{CreatedComponent, SelectedComponent},
    DeselectQuery, EditorState, SelectQuery,
};

pub fn selector(
    state: Res<CurrentState<EditorState>>,
    mut events: EventReader<PickingEvent>,
    mut commands: Commands,
    deselect_query: DeselectQuery,
    buttons: Res<Input<MouseButton>>,
) {
    if matches!(&state.0, EditorState::CreatingComponent(_)) {
        return;
    }
    if buttons.just_released(MouseButton::Left) {
        let mut clicked = false;
        for event in events.iter() {
            if let PickingEvent::Clicked(e) = event {
                clicked = true;
                deselect(&mut commands, &deselect_query);
                commands.entity(*e).insert(SelectedComponent);
            }
        }
        if !clicked {
            deselect(&mut commands, &deselect_query);
        }
    }
}

pub fn deselect(commands: &mut Commands, (selected_query, shadow_query, skin): &DeselectQuery) {
    for (data, coords, entity) in selected_query.iter() {
        commands
            .entity(entity)
            .insert_bundle(data.get_shape(coords.to_owned(), skin, false))
            .remove::<SelectedComponent>();
    }
    for entity in shadow_query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn select(commands: &mut Commands, set: &mut SelectQuery<impl WorldQuery>) {
    if !set.p1().0.is_empty() {
        deselect(commands, &set.p0())
    }
    let (select_query, skin) = set.p1();
    for (data, coords, entity) in select_query.iter() {
        commands
            .entity(entity)
            .insert_bundle(data.get_shape(coords.to_owned(), &skin, false))
            .remove::<CreatedComponent>()
            .insert(SelectedComponent);
        commands.spawn_bundle(SelectShadowBundle {
            _marker: SelectShadow,
            shape: data.get_shape(coords.to_owned(), &skin, true),
        });
    }
}
