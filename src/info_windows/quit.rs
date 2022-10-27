use std::sync::Arc;

use bevy::{app::AppExit, prelude::*};

use crate::{
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
        if event.id == "quit" {
            if components.is_empty() || cfg!(debug_assertions) {
                send_queue.push(Action::new("quit1"));
            } else {
                popup.send(Popup::base_confirm(
                    "quit1",
                    "Are you sure you want to exit?",
                    "You may have unsaved changes",
                    (),
                ))
            };
        } else if event.id == "quit1" {
            exit.send(AppExit)
        }
    }
    for action in send_queue {
        actions.p1().send(action)
    }
}
