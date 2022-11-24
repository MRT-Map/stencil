use bevy::prelude::*;
use iyes_loopless::prelude::*;
use native_dialog::{MessageDialog, MessageType};

use crate::{
    component_actions::undo_redo::{History, UndoRedoAct},
    cursor::mouse_events::MouseEvent,
    misc::{Action, EditorState},
    pla2::component::{EditorCoords, PlaComponent},
};

#[tracing::instrument(skip_all)]
pub fn delete_component_sy(
    mut mouse: EventReader<MouseEvent>,
    mut commands: Commands,
    query: Query<(&PlaComponent<EditorCoords>, Entity)>,
    mut actions: EventWriter<Action>,
) {
    for event in mouse.iter() {
        if let MouseEvent::LeftClick(Some(e), _) = event {
            let pla = query.iter().find(|(_, a)| a == e).unwrap().0;
            if pla.nodes.len() > 5 && !MessageDialog::default() // TODO remove this when undo/redo is implemented
                    .set_title("This component has more than 5 nodes, are you sure you want to delete?")
                    .set_text("We have not implemented undoing and redoing yet so your progress may be lost!")
                    .set_type(MessageType::Warning)
                    .show_confirm().unwrap() {
                continue
            }
            info!(?e, "Deleting entity");
            actions.send(Box::new(UndoRedoAct::one_history(History {
                component_id: *e,
                before: Some(pla.to_owned()),
                after: None,
            })));
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
