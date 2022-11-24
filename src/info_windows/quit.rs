use std::sync::Arc;

use bevy::{app::AppExit, prelude::*};

use crate::{
    info_windows::InfoWindowsAct,
    misc::Action,
    pla2::component::{EditorCoords, PlaComponent},
    ui::popup::Popup,
};

pub fn quit_asy(
    mut actions: ParamSet<(EventReader<Action>, EventWriter<Action>)>,
    mut popup: EventWriter<Arc<Popup>>,
    mut exit: EventWriter<AppExit>,
    components: Query<(), With<PlaComponent<EditorCoords>>>,
) {
    let mut send_queue: Vec<Action> = vec![];
    for event in actions.p0().iter() {
        if let Some(InfoWindowsAct::Quit(false)) = event.downcast_ref() {
            if components.is_empty() || cfg!(debug_assertions) {
                send_queue.push(Box::new(InfoWindowsAct::Quit(true)));
            } else {
                popup.send(Popup::base_confirm(
                    "confirm_quit",
                    "Are you sure you want to exit?",
                    "You may have unsaved changes",
                    InfoWindowsAct::Quit(true),
                ))
            };
        } else if let Some(InfoWindowsAct::Quit(true)) = event.downcast_ref() {
            {
                exit.send(AppExit)
            }
        }
    }

    for action in send_queue {
        actions.p1().send(action)
    }
}
