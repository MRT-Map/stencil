use bevy::prelude::*;

use crate::{
    component::{
        actions::rendering::RenderEv,
        pla2::{EditorCoords, PlaComponent},
    },
    state::EditorState,
    ui::{
        cursor::mouse_events::Click2, panel::status::Status, tilemap::window::PointerWithinTilemap,
    },
};

#[tracing::instrument(skip_all)]
pub fn on_select_left_click(
    trigger: Trigger<Pointer<Click2>>,
    mut commands: Commands,
    state: Res<State<EditorState>>,
    components: Query<(), With<PlaComponent<EditorCoords>>>,
    mut status: ResMut<Status>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none()
        || state.component_type().is_some()
        || *state == EditorState::DeletingComponent
        || trigger.button != PointerButton::Primary
    {
        return;
    }

    let e = trigger.entity();
    if e == Entity::PLACEHOLDER {
        info!("Selected nothing, deselecting");
        commands.trigger(SelectEv::DeselectAll);
        status.0 = "Deselected component".into();
    } else if components.contains(e) {
        commands.trigger_targets(SelectEv::SelectOne, e);
        status.0 = "Selected component".into();
    }
}

#[tracing::instrument(skip_all)]
pub fn on_select(
    trigger: Trigger<SelectEv>,
    mut commands: Commands,
    mut query: ParamSet<(
        Query<&PlaComponent<EditorCoords>>,
        Query<Entity, With<SelectedComponent>>,
    )>,
) {
    let e = trigger.entity();
    if e == Entity::PLACEHOLDER && *trigger.event() != SelectEv::DeselectAll {
        return;
    }
    match trigger.event() {
        SelectEv::Select => {
            info!(?e, "Selecting entity");
            commands
                .entity(e)
                .insert(SelectedComponent)
                .trigger(RenderEv::default());
        }
        SelectEv::Deselect => {
            debug!(?e, "Deselecting component");
            commands
                .entity(e)
                .remove::<SelectedComponent>()
                .trigger(RenderEv::default());
        }
        SelectEv::SelectOne => {
            commands.trigger(SelectEv::DeselectAll);
            commands.trigger_targets(SelectEv::Select, e);
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
        app.add_observer(on_select_left_click)
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

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedComponent;
