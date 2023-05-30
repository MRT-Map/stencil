use bevy::prelude::{Commands, EventReader, EventWriter, NextState, ParamSet, Res, States};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    component_tools::creating::{clear_created_component, CreatedQuery},
    misc::Action,
    pla2::{component::ComponentType, skin::Skin},
    ui::panel::component_panel::PrevNamespaceUsed,
};

#[derive(States, Deserialize, Serialize, Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditorState {
    #[default]
    Loading,
    Idle,
    CreatingPoint,
    CreatingLine,
    CreatingArea,
    EditingNodes,
    DeletingComponent,
}

impl EditorState {
    #[must_use]
    pub const fn component_type(self) -> Option<ComponentType> {
        match self {
            Self::CreatingArea => Some(ComponentType::Area),
            Self::CreatingLine => Some(ComponentType::Line),
            Self::CreatingPoint => Some(ComponentType::Point),
            _ => None,
        }
    }
}

#[derive(States, Deserialize, Serialize, Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum LoadingState {
    #[default]
    SetIcon,
    UnzipAssets,
    LoadAssets,
    Compat,
    LoadSkin,
    SpawnCamera,
    Done,
}
impl LoadingState {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::SetIcon => Self::UnzipAssets,
            Self::UnzipAssets => Self::LoadAssets,
            Self::LoadAssets => Self::Compat,
            Self::Compat => Self::LoadSkin,
            Self::LoadSkin => Self::SpawnCamera,
            Self::SpawnCamera => Self::Done,
            Self::Done => unreachable!(),
        }
    }
}

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
        commands.insert_resource(NextState(Some(state)));
    }
}
