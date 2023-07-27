use bevy::{app::AppExit, prelude::*};

use crate::{
    info_windows::InfoWindowsAct,
    misc::Action,
    pla2::component::{EditorCoords, PlaComponent},
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
    for event in &mut actions.p0() {
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
