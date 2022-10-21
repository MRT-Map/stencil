use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use iyes_loopless::prelude::*;

use crate::{
    editor::{bundles::component::SelectedComponent, cursor::mouse_events::MouseEvent},
    types::{
        pla::{EditorCoords, PlaComponent},
        skin::Skin,
        DeselectQuery, EditorState,
    },
};

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all)]
pub fn selector_sy(
    mut commands: Commands,
    state: Res<CurrentState<EditorState>>,
    mut events: EventReader<MouseEvent>,
    deselect_query: DeselectQuery,
) {
    if matches!(
        &state.0,
        EditorState::CreatingComponent(_) | EditorState::DeletingComponent
    ) {
        return;
    }
    for event in events.iter() {
        if let MouseEvent::LeftClick(e, _) = event {
            if let Some(e) = e {
                select_entity(&mut commands, &deselect_query, e);
            } else {
                info!("Selected nothing, deselecting");
                deselect(&mut commands, &deselect_query);
            }
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn highlight_selected_sy(
    state: Res<CurrentState<EditorState>>,
    mut commands: Commands,
    query: Query<(&PlaComponent<EditorCoords>, Entity), Changed<SelectedComponent>>,
    skin: Res<Skin>,
) {
    if matches!(&state.0, EditorState::CreatingComponent(_)) {
        return;
    }
    for (data, entity) in query.iter() {
        trace!(?entity, "Highlighting selected component");
        commands
            .entity(entity)
            .insert_bundle(data.get_shape(&skin, true));
    }
}

pub fn deselect(commands: &mut Commands, (selected_query, skin): &DeselectQuery) {
    for (data, entity) in selected_query.iter() {
        debug!(?entity, "Deselecting component");
        commands
            .entity(entity)
            .remove::<SelectedComponent>()
            .remove_bundle::<ShapeBundle>()
            .insert_bundle(data.get_shape(skin, false))
            .despawn_descendants();
    }
}

pub fn select_entity(commands: &mut Commands, deselect_query: &DeselectQuery, entity: &Entity) {
    info!(?entity, "Selecting entity");
    deselect(commands, deselect_query);
    commands.entity(*entity).insert(SelectedComponent);
}

pub struct SelectComponentPlugin;
impl Plugin for SelectComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .after("highlight_selected")
                //.after(PickingSystem::Events)
                .with_system(selector_sy)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .label("highlight_selected")
                .run_not_in_state(EditorState::Loading)
                .with_system(highlight_selected_sy)
                .into(),
        );
    }
}
