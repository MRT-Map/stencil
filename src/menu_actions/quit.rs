use bevy::{app::AppExit, prelude::*};
use native_dialog::{MessageDialog, MessageType};

use crate::{
    action,
    misc::Action,
    pla2::component::{EditorCoords, PlaComponent},
};

pub fn quit_msy(
    mut events: EventReader<Action>,
    mut exit: EventWriter<AppExit>,
    components: Query<(), With<PlaComponent<EditorCoords>>>,
) {
    action!(events, "quit", |_| {
        if components.is_empty()
            || cfg!(debug_assertions)
            || MessageDialog::default()
                .set_title("Are you sure you want to exit?")
                .set_text("You may have unsaved changes")
                .set_type(MessageType::Warning)
                .show_confirm()
                .unwrap()
        {
            exit.send(AppExit);
        }
    });
}
