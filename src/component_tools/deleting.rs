use bevy::prelude::*;
use iyes_loopless::prelude::*;
use native_dialog::{MessageDialog, MessageType};

use crate::{
    cursor::mouse_events::MouseEvent,
    pla2::component::{EditorCoords, PlaComponent},
    setup::EditorState,
};

#[tracing::instrument(skip_all)]
pub fn delete_component_sy(
    mut events: EventReader<MouseEvent>,
    mut commands: Commands,
    query: Query<(&PlaComponent<EditorCoords>, Entity)>,
    _non_send: Option<NonSend<()>>,
) {
    for event in events.iter() {
        if let MouseEvent::LeftClick(Some(e), _) = event {
            let pla: &PlaComponent<EditorCoords> = query.iter().find(|(_, a)| a == e).unwrap().0;
            if pla.nodes.len() > 5 && !MessageDialog::default()
                    .set_title("This component has more than 5 nodes, are you sure you want to delete?")
                    .set_text("We have not implemented undoing and redoing yet so your progress may be lost!")
                    .set_type(MessageType::Warning)
                    .show_confirm().unwrap() {
                continue
            }
            info!(?e, "Deleting entity");
            commands.entity(*e).despawn_recursive();
        }
    }
}

pub struct DeleteComponentPlugin;

impl Plugin for DeleteComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(delete_component_sy.run_in_state(EditorState::DeletingComponent));
    }
}
