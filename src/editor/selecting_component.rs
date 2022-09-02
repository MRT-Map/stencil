use bevy::prelude::*;
use bevy_mod_picking::PickingEvent;
use iyes_loopless::prelude::*;

use crate::{
    pla::{ComponentCoords, EditorComponent, SelectedComponent},
    EditorState, Skin,
};

pub fn selector(
    state: Res<CurrentState<EditorState>>,
    mut events: EventReader<PickingEvent>,
    mut commands: Commands,
    selected_query: Query<(&EditorComponent, &ComponentCoords, Entity), With<SelectedComponent>>,
    skin: Res<Skin>,
    buttons: Res<Input<MouseButton>>,
) {
    if let EditorState::CreatingComponent(_) = &state.0 {
        return;
    }
    if buttons.just_released(MouseButton::Left) {
        let mut clicked = false;
        for event in events.iter() {
            if let PickingEvent::Clicked(e) = event {
                clicked = true;
                for (data, coords, entity) in selected_query.iter() {
                    commands
                        .entity(entity)
                        .insert_bundle(data.get_shape(coords.to_owned(), &skin))
                        .remove::<SelectedComponent>();
                }
                commands.entity(*e).insert(SelectedComponent);
            }
        }
        if !clicked {
            for (data, coords, entity) in selected_query.iter() {
                commands
                    .entity(entity)
                    .insert_bundle(data.get_shape(coords.to_owned(), &skin))
                    .remove::<SelectedComponent>();
            }
        }
    }
}
