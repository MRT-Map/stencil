use bevy::prelude::*;

use crate::{misc::data_path, state::LoadingState, ui::popup::Popup};

#[allow(clippy::needless_pass_by_value)]
pub fn welcome_sy(mut commands: Commands, mut popup: EventWriter<Popup>) {
    if !data_path(".welcome_shown").exists() {
        popup.send(Popup::base_alert(
            "welcome",
            "Welcome to Stencil!",
            "Remember to read our manual at https://github.com/MRT-Map/stencil2/wiki. \n\n\
                If you have any questions not covered in the wiki, or you have found a bug, \
                feel free to ask for help on the MRT Mapping Services server (if you have access), \
                contact __7d (if you know my Discord account), or open an issue on our GitHub page \
                at https://github.com/MRT-Map/stencil2/issues.",
        ));
        let _ = std::fs::write(data_path(".welcome_shown"), "");
    }

    commands.insert_resource(NextState(Some(LoadingState::Welcome.next())));
}
