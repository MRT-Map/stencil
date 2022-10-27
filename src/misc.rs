use std::{any::Any, fmt::Display};

use bevy::prelude::*;
use iyes_loopless::prelude::NextState;

use crate::pla2::component::ComponentType;

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditorState {
    #[default]
    Loading,
    Idle,
    CreatingComponent(ComponentType),
    EditingNodes,
    DeletingComponent,
}

pub struct Action<P: Send + Sync + ?Sized = dyn Any + Send + Sync> {
    pub id: String,
    pub payload: Box<P>,
}
impl Action {
    pub fn new(id: impl Display) -> Self {
        Self {
            id: id.to_string(),
            payload: Box::new(()),
        }
    }
}

pub fn state_changer_asy(mut commands: Commands, mut actions: EventReader<Action>) {
    for event in actions.iter() {
        if event.id == "change_state" {
            let state: &EditorState = event.payload.downcast_ref().unwrap();
            commands.insert_resource(NextState(*state))
        }
    }
}
