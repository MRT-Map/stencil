use bevy::prelude::*;

use crate::{
    component::pla2::{EditorCoords, PlaComponent},
    history::{HistoryEntry, HistoryEv},
    state::EditorState,
    ui::panel::status::Status,
};

#[tracing::instrument(skip_all)]
pub fn delete_component_sy(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    query: Query<(&PlaComponent<EditorCoords>)>,
    mut status: ResMut<Status>,
    state: Res<State<EditorState>>,
) {
    if **state != EditorState::DeletingComponent {
        return;
    }
    let entity = trigger.entity();
    let Ok(pla) = query.get(entity) else {
        return;
    };
    info!(?entity, "Deleting entity");
    commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
        entity,
        before: Some(pla.to_owned().into()),
        after: None,
    }));
    commands.entity(entity).despawn_recursive();
    status.0 = format!("Deleted {pla}").into();
}

pub struct DeleteComponentPlugin;

impl Plugin for DeleteComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(delete_component_sy);
    }
}
