use bevy::prelude::*;

use crate::{
    component::pla2::PlaComponent,
    history::{HistoryEntry, HistoryEv},
    state::EditorState,
    ui::{cursor::mouse_events::Click2, map::window::PointerWithinTilemap, panel::status::Status},
};

#[tracing::instrument(skip_all)]
pub fn delete_component_sy(
    trigger: Trigger<Pointer<Click2>>,
    mut commands: Commands,
    state: Res<State<EditorState>>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none() || **state != EditorState::DeletingComponent {
        return;
    }
    commands.entity(trigger.target()).trigger(DeleteEv);
}

#[tracing::instrument(skip_all)]
pub fn on_delete(
    trigger: Trigger<DeleteEv>,
    mut commands: Commands,
    query: Query<&PlaComponent>,
    mut status: ResMut<Status>,
) {
    let e = trigger.target();
    let Ok(pla) = query.get(e) else {
        return;
    };

    info!(?e, "Deleting entity");
    commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
        e,
        before: Some(pla.to_owned().into()),
        after: None,
    }));
    commands.entity(e).despawn();
    status.set(format!("Deleted {pla}"));
}

pub struct DeleteComponentPlugin;

impl Plugin for DeleteComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(delete_component_sy)
            .add_observer(on_delete);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Event)]
pub struct DeleteEv;
