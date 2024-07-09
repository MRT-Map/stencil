use bevy::{
    ecs::schedule::{SystemConfigs, SystemSetConfigs},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    component::{
        pla2::ComponentType,
        skin::Skin,
        tools::creating::{clear_created_component, CreatedQuery},
    },
    project::Namespaces,
    ui::panel::status::Status,
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
    Welcome,
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
            Self::SpawnCamera => Self::Welcome,
            Self::Welcome => Self::Done,
            Self::Done => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Event)]
pub struct ChangeStateAct(pub EditorState);

#[allow(clippy::needless_pass_by_value)]
pub fn on_state_change(
    trigger: Trigger<ChangeStateAct>,
    mut commands: Commands,
    mut created_query: CreatedQuery,
    skin: Res<Skin>,
    mut namespaces: ResMut<Namespaces>,
    mut status: ResMut<Status>,
    state: Res<State<EditorState>>,
) {
    if trigger.event().0 == **state {
        return;
    }
    info!(?state, "Changing state");
    clear_created_component(
        &mut commands,
        &mut created_query,
        &skin,
        &mut namespaces,
        &mut status,
        "component",
    );
    commands.insert_resource(NextState::Pending(trigger.event().0));
}

pub trait IntoSystemConfigExt<Marker>: IntoSystemConfigs<Marker> {
    fn run_if_not_loading(self) -> SystemConfigs {
        self.into_configs()
            .run_if(not(in_state(EditorState::Loading)))
    }
}

impl<T, Marker> IntoSystemConfigExt<Marker> for T where T: IntoSystemConfigs<Marker> {}

pub trait IntoSystemSetConfigExt: IntoSystemSetConfigs {
    fn run_if_not_loading(self) -> SystemSetConfigs {
        self.into_configs()
            .run_if(not(in_state(EditorState::Loading)))
    }
}

impl<T> IntoSystemSetConfigExt for T where T: IntoSystemSetConfigs {}
