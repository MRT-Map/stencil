use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;

use crate::{
    component::{
        bundle::{EntityCommandsSelectExt, SelectedComponent},
        circle::circle,
        pla2::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
    state::{EditorState, IntoSystemConfigExt},
    tile::zoom::Zoom,
    ui::{cursor::mouse_events::MouseEvent, panel::status::Status, UiSet},
};

#[tracing::instrument(skip_all)]
pub fn selector_sy(
    mut commands: Commands,
    state: Res<State<EditorState>>,
    mut mouse: EventReader<MouseEvent>,
    deselect_query: DeselectQuery,
    mut status: ResMut<Status>,
) {
    if state.component_type().is_some() || *state == EditorState::DeletingComponent {
        mouse.clear();
        return;
    }
    for event in mouse.read() {
        if let MouseEvent::LeftClick(e, _) = event {
            if let Some(e) = e {
                select_entity(&mut commands, &deselect_query, *e);
                status.0 = "Selected component".into();
            } else {
                info!("Selected nothing, deselecting");
                deselect(&mut commands, &deselect_query);
                status.0 = "Deselected component".into();
            }
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn highlight_selected_sy(
    state: Res<State<EditorState>>,
    mut commands: Commands,
    query: Query<(&PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    skin: Res<Skin>,
    zoom: Res<Zoom>,
) {
    if state.component_type().is_some() {
        return;
    }
    for (data, entity) in query.iter() {
        trace!(?entity, "Highlighting selected component");
        commands.entity(entity).select_component(&skin, data);
        if data.get_type(&skin) == Some(ComponentType::Line) && !data.nodes.is_empty() {
            commands.entity(entity).despawn_descendants();
            let start = commands
                .spawn(circle(
                    data,
                    &skin,
                    &zoom,
                    data.nodes.first().unwrap().0.as_vec2(),
                    1.0,
                    Color::GREEN,
                ))
                .id();
            let end = commands
                .spawn(circle(
                    data,
                    &skin,
                    &zoom,
                    data.nodes.last().unwrap().0.as_vec2(),
                    1.0,
                    Color::RED,
                ))
                .id();
            commands.entity(entity).add_child(start).add_child(end);
        }
    }
}

pub fn deselect(commands: &mut Commands, (selected_query, skin): &DeselectQuery) {
    for (data, entity) in selected_query.iter() {
        debug!(?entity, "Deselecting component");
        commands
            .entity(entity)
            .remove::<SelectedComponent>()
            .remove::<ShapeBundle>()
            .component_display(skin, data)
            .despawn_descendants();
    }
}

pub fn select_entity(commands: &mut Commands, deselect_query: &DeselectQuery, entity: Entity) {
    info!(?entity, "Selecting entity");
    deselect(commands, deselect_query);
    commands.entity(entity).insert(SelectedComponent);
}

pub struct SelectComponentPlugin;
impl Plugin for SelectComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, selector_sy.run_if_not_loading())
            .add_systems(
                PreUpdate,
                highlight_selected_sy
                    .run_if_not_loading()
                    .after(UiSet::Reset),
            );
    }
}

pub type DeselectQuery<'world, 'state, 'a> = (
    Query<'world, 'state, (&'a PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    Res<'world, Skin>,
);
