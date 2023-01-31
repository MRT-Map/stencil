use std::{
    any::Any,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use iyes_loopless::prelude::NextState;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    component_tools::creating::{clear_created_component, CreatedQuery},
    pla2::{component::ComponentType, skin::Skin},
    ui::component_panel::PrevNamespaceUsed,
};

pub static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("stencil2");
    let _ = std::fs::create_dir_all(&dir);
    dir
});

pub fn data_dir(next: impl AsRef<Path>) -> PathBuf {
    let path = DATA_DIR.join(next);
    let _ = std::fs::create_dir_all(&path);
    path
}

#[derive(Deserialize, Serialize, Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
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

#[allow(clippy::needless_pass_by_value)]
pub fn state_changer_asy(
    mut commands: Commands,
    mut actions: ParamSet<(EventReader<Action>, EventWriter<Action>)>,
    mut created_query: CreatedQuery,
    skin: Res<Skin>,
    prev_namespace_used: Res<PrevNamespaceUsed>,
) {
    let mut new_state = None;
    let mut reader = actions.p0();
    for event in reader.iter() {
        if let Some(ChangeStateAct(state)) = event.downcast_ref() {
            new_state = Some(*state);
        }
    }
    if let Some(state) = new_state {
        info!(?state, "Changing state");
        let mut writer = actions.p1();
        clear_created_component(
            &mut commands,
            &mut created_query,
            &skin,
            &prev_namespace_used.0,
            &mut writer,
        );
        commands.insert_resource(NextState(state));
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
