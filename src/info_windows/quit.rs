use bevy::{app::AppExit, prelude::*};

use crate::{
    action::Action,
    component::pla2::{EditorCoords, PlaComponent},
    info_windows::InfoWindowsAct,
    ui::popup::Popup,
};

#[allow(clippy::needless_pass_by_value)]
pub fn quit_asy(
    mut actions: ParamSet<(EventReader<Action>, EventWriter<Action>)>,
    mut popup: EventWriter<Popup>,
    mut exit: EventWriter<AppExit>,
    components: Query<(), With<PlaComponent<EditorCoords>>>,
) {
    let mut send_queue: Vec<Action> = vec![];
    for event in actions.p0().read() {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Quit(false))) {
            if components.is_empty() || cfg!(debug_assertions) {
                send_queue.push(Action::new(InfoWindowsAct::Quit(true)));
            } else {
                popup.send(Popup::base_confirm(
                    "confirm_quit",
                    "Are you sure you want to exit?",
                    "You may have unsaved changes",
                    Action::new(InfoWindowsAct::Quit(true)),
                ));
            };
        } else if matches!(event.downcast_ref(), Some(InfoWindowsAct::Quit(true))) {
            {
                exit.send(AppExit);
            }
        }
    }

    for action in send_queue {
        actions.p1().send(action);
    }
}
