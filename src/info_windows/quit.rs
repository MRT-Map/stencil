use bevy::{app::AppExit, prelude::*};

use crate::{
    component::pla2::{EditorCoords, PlaComponent},
    info_windows::InfoWindowsEv,
    ui::popup::Popup,
};

#[allow(clippy::needless_pass_by_value)]
pub fn on_quit(
    trigger: Trigger<InfoWindowsEv>,
    mut popup: EventWriter<Popup>,
    mut exit: EventWriter<AppExit>,
    components: Query<(), With<PlaComponent<EditorCoords>>>,
    mut commands: Commands,
) {
    match trigger.event() {
        InfoWindowsEv::Quit(false) => {
            if components.is_empty() || cfg!(debug_assertions) {
                commands.trigger(InfoWindowsEv::Quit(true));
            } else {
                popup.send(Popup::base_confirm(
                    "confirm_quit",
                    "Are you sure you want to exit?",
                    "You may have unsaved changes",
                    InfoWindowsEv::Quit(true),
                ));
            };
        }
        InfoWindowsEv::Quit(true) => {
            exit.send(AppExit::Success);
        }
        _ => {}
    }
}
