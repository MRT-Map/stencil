use bevy::{app::AppExit, prelude::*};
use native_dialog::{MessageDialog, MessageType};

use crate::{
    menu,
    menu_actions::MenuAction,
    pla2::component::{EditorCoords, PlaComponent},
};

pub fn quit_msy(
    mut events: EventReader<MenuAction>,
    mut exit: EventWriter<AppExit>,
    components: Query<(), With<PlaComponent<EditorCoords>>>,
) {
    menu!(events, "quit");
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
}
