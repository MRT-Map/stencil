use bevy::prelude::*;

use crate::{
    component::pla2::{EditorCoords, PlaComponent},
    history::{HistoryEntry, HistoryEv},
    state::EditorState,
    ui::panel::status::Status,
};
use crate::ui::cursor::mouse_events::Click2;
use crate::ui::tilemap::window::PointerWithinTilemap;

#[tracing::instrument(skip_all)]
pub fn delete_component_sy(
    trigger: Trigger<Pointer<Click2>>,
    mut commands: Commands,
    query: Query<&PlaComponent<EditorCoords>>,
    mut status: ResMut<Status>,
    state: Res<State<EditorState>>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) {
    if pointer_within_tilemap.is_none() || **state != EditorState::DeletingComponent {
        return;
    }
    let e = trigger.entity();
    let Ok(pla) = query.get(e) else {
        return;
    };
    info!(?e, "Deleting entity");
    commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
        e,
        before: Some(pla.to_owned().into()),
        after: None,
    }));
    commands.entity(e).despawn_recursive();
    status.0 = format!("Deleted {pla}").into();
}

pub struct DeleteComponentPlugin;

impl Plugin for DeleteComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(delete_component_sy);
    }
}
