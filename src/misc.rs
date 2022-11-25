use std::{any::Any, path::PathBuf};

use bevy::prelude::*;
use iyes_loopless::prelude::NextState;
use once_cell::sync::Lazy;

use crate::pla2::component::ComponentType;

pub static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let mut dir = dirs::data_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    dir.push("stencil2");
    let _ = std::fs::create_dir(&dir);
    dir
});

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditorState {
    #[default]
    Loading,
    Idle,
    CreatingComponent(ComponentType),
    EditingNodes,
    DeletingComponent,
}

pub type Action = Box<dyn Any + Send + Sync>;

pub struct ChangeStateAct(pub EditorState);

pub fn state_changer_asy(mut commands: Commands, mut actions: EventReader<Action>) {
    for event in actions.iter() {
        if let Some(ChangeStateAct(state)) = event.downcast_ref() {
            info!(?state, "Changing state");
            commands.insert_resource(NextState(*state))
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CustomStage {
    Ui,
    Cursor,
}
impl StageLabel for CustomStage {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Cursor => "cursor",
        }
    }
}
