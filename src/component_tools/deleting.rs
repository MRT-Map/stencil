use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{cursor::mouse_events::MouseEvent, setup::EditorState};

#[tracing::instrument(skip_all)]
pub fn delete_component_sy(mut events: EventReader<MouseEvent>, mut commands: Commands) {
    for event in events.iter() {
        if let MouseEvent::LeftClick(Some(e), _) = event {
            info!(?e, "Deleting entity");
            commands.entity(*e).despawn_recursive();
        }
    }
}

pub struct DeleteComponentPlugin;

impl Plugin for DeleteComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(EditorState::DeletingComponent)
                .with_system(delete_component_sy)
                .into(),
        );
    }
}
