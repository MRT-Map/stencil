use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_mod_picking::PickingEvent;
use bevy_prototype_lyon::entity::ShapeBundle;
use iyes_loopless::prelude::*;

use crate::{
    types::{
        DeselectQuery, EditorState, SelectQuery,
    },
};
use crate::editor::bundles::component::{CreatedComponent, EditorComponent, SelectedComponent};
use crate::editor::ui::HoveringOverGui;
use crate::types::ComponentType;
use crate::types::pla::ComponentCoords;
use crate::types::skin::Skin;

pub fn selector(
    state: Res<CurrentState<EditorState>>,
    mut events: EventReader<PickingEvent>,
    mut commands: Commands,
    deselect_query: DeselectQuery,
    buttons: Res<Input<MouseButton>>,
    hovering: Res<HoveringOverGui>,
) {
    if matches!(&state.0, EditorState::CreatingComponent(_)) {
        return;
    }
    let mut clicked = false;
    for event in events.iter() {
        println!("{:#?}", event);
        if let PickingEvent::Clicked(e) = event {
            clicked = true;
            select_entity(&mut commands, &deselect_query, e);
        }
    }
    if !clicked && buttons.just_pressed(MouseButton::Left) && !hovering.0 {
        deselect(&mut commands, &deselect_query);
    }
}

pub fn highlight_selected(
    mut commands: Commands,
    query: Query<(&EditorComponent, &ComponentCoords, Entity), With<SelectedComponent>>,
    skin: Res<Skin>
) {
    for (data, coords, entity) in query.iter() {
        commands.entity(entity)
            .insert_bundle(data.get_shape(coords.to_owned(), &skin, true));
    }
}

pub fn deselect(commands: &mut Commands, (selected_query, skin): &DeselectQuery) {
    for (data, coords, entity) in selected_query.iter() {
        commands
            .entity(entity)
            .remove_bundle::<ShapeBundle>()
            .insert_bundle(data.get_shape(coords.to_owned(), skin, false))
            .remove::<SelectedComponent>();
    }
}

pub fn select_entity(commands: &mut Commands, deselect_query: &DeselectQuery, entity: &Entity) {
    deselect(commands, deselect_query);
    commands.entity(*entity).insert(SelectedComponent);
}

pub fn select_query(commands: &mut Commands, set: &mut SelectQuery<impl WorldQuery>) {
    if !set.p1().is_empty() {
        deselect(commands, &set.p0())
    }
    let query = set.p1();
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<CreatedComponent>()
            .insert(SelectedComponent);
    }
}

pub struct SelectComponentPlugin;
impl Plugin for SelectComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .run_not_in_state(EditorState::CreatingComponent(ComponentType::Point))
                .run_not_in_state(EditorState::CreatingComponent(ComponentType::Line))
                .run_not_in_state(EditorState::CreatingComponent(ComponentType::Area))
                .with_system(selector)
                .with_system(highlight_selected)
                .into(),
        );
    }
}

