use bevy::prelude::*;

use crate::{
    component::pla2::{EditorCoords, PlaComponent},
    history::{HistoryAct, HistoryEntry},
    state::EditorState,
    ui::{cursor::mouse_events::MouseEvent, panel::status::Status},
};

#[tracing::instrument(skip_all)]
pub fn delete_component_sy(
    mut mouse: EventReader<MouseEvent>,
    mut commands: Commands,
    query: Query<(&PlaComponent<EditorCoords>, Entity)>,
    mut status: ResMut<Status>,
) {
    for event in mouse.read() {
        if let MouseEvent::LeftClick(Some(e), _) = event {
            let (pla, _) = query.iter().find(|(_, a)| a == e).unwrap();
            info!(?e, "Deleting entity");
            commands.trigger(HistoryAct::one_history(HistoryEntry::Component {
                entity: *e,
                before: Some(pla.to_owned().into()),
                after: None,
            }));
            commands.entity(*e).despawn_recursive();
            status.0 = format!("Deleted {pla}").into();
        }
    }
}

pub struct DeleteComponentPlugin;

impl Plugin for DeleteComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            delete_component_sy.run_if(in_state(EditorState::DeletingComponent)),
        );
    }
}
