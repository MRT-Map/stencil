use bevy::{
    color::palettes::basic::{LIME, RED},
    prelude::*,
};
use bevy_prototype_lyon::entity::ShapeBundle;

use crate::{
    component::{
        bundle::{EntityCommandsSelectExt, SelectedComponent},
        circle::circle,
        pla2::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
    misc_config::settings::MiscSettings,
    state::EditorState,
    tile::zoom::Zoom,
    ui::panel::status::Status,
};
use crate::state::IntoSystemConfigExt;
use crate::ui::panel::dock::PanelDockState;
use crate::ui::UiSet;

#[tracing::instrument(skip_all)]
pub fn on_select_left_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    state: Res<State<EditorState>>,
    components: Query<(), With<PlaComponent<EditorCoords>>>,
    mut status: ResMut<Status>,
    panel: Res<PanelDockState>,
) {
    if !panel.pointer_within_tilemap || state.component_type().is_some()
        || *state == EditorState::DeletingComponent
        || trigger.button != PointerButton::Primary
    {
        return;
    }
    let entity = trigger.entity();
    if entity != Entity::PLACEHOLDER && !components.contains(trigger.entity()) {
        return;
    }

    if entity == Entity::PLACEHOLDER {
        info!("Selected nothing, deselecting");
        commands.trigger(SelectEv::DeselectAll);
        status.0 = "Deselected component".into();
    } else {
        commands.trigger_targets(SelectEv::SelectOne, trigger.entity());
        status.0 = "Selected component".into();
    }
}

#[tracing::instrument(skip_all)]
pub fn highlight_selected_sy(
    state: Res<State<EditorState>>,
    mut commands: Commands,
    query: Query<(&PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    skin: Res<Skin>,
    zoom: Res<Zoom>,
    misc_settings: Res<MiscSettings>,
) {
    if state.component_type().is_some() {
        return;
    }
    for (data, entity) in query.iter() {
        if data.get_type(&skin) == ComponentType::Line && !data.nodes.is_empty() {
            commands.entity(entity).despawn_descendants();
            let start = commands
                .spawn(circle(
                    &zoom,
                    data.nodes.first().unwrap().0.as_vec2(),
                    misc_settings.big_handle_size,
                    LIME.into(),
                ))
                .id();
            let end = commands
                .spawn(circle(
                    &zoom,
                    data.nodes.last().unwrap().0.as_vec2(),
                    misc_settings.big_handle_size,
                    RED.into(),
                ))
                .id();
            commands.entity(entity).add_child(start).add_child(end);
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn on_select(
    trigger: Trigger<SelectEv>,
    mut commands: Commands,
    skin: Res<Skin>,
    mut query: ParamSet<(
        Query<&PlaComponent<EditorCoords>>,
        Query<Entity, With<SelectedComponent>>,
    )>,
) {
    let entity = trigger.entity();
    if entity == Entity::PLACEHOLDER && *trigger.event() != SelectEv::DeselectAll {
        return;
    }
    match trigger.event() {
        SelectEv::Select => {
            info!(?entity, "Selecting entity");
            commands.entity(entity).insert(SelectedComponent);
            commands.entity(entity).select_component(&skin, query.p0().get(entity).unwrap());
        }
        SelectEv::Deselect => {
            debug!(?entity, "Deselecting component");
            commands
                .entity(entity)
                .remove::<SelectedComponent>()
                .remove::<ShapeBundle>()
                .component_display(&skin, query.p0().get(entity).unwrap())
                .despawn_descendants();
        }
        SelectEv::SelectOne => {
            commands.trigger(SelectEv::DeselectAll);
            commands.trigger_targets(SelectEv::Select, entity);
        }
        SelectEv::DeselectAll => {
            commands.trigger_targets(
                SelectEv::Deselect,
                query
                    .p1()
                    .iter()
                    .filter(|a| *a != Entity::PLACEHOLDER)
                    .collect::<Vec<_>>(),
            );
        }
    }
}

pub struct SelectComponentPlugin;
impl Plugin for SelectComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            highlight_selected_sy
                .run_if_not_loading()
                .after(UiSet::Reset),
        )
        .add_observer(on_select_left_click)
        .add_observer(on_select);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Event)]
pub enum SelectEv {
    Select,
    Deselect,
    SelectOne,
    DeselectAll,
}
