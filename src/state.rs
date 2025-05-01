use bevy::{
    ecs::schedule::{graph::GraphInfo, Chain, Schedulable, ScheduleConfigs},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::component::{pla2::ComponentType, tools::creating::ClearCreatedComponentEv};

#[derive(States, Deserialize, Serialize, Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
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
    LoadFonts,
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
            Self::LoadSkin => Self::LoadFonts,
            Self::LoadFonts => Self::SpawnCamera,
            Self::SpawnCamera => Self::Welcome,
            Self::Welcome => Self::Done,
            Self::Done => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Event)]
pub struct ChangeStateEv(pub EditorState);

#[expect(clippy::needless_pass_by_value)]
pub fn on_state_change(
    trigger: Trigger<ChangeStateEv>,
    mut commands: Commands,
    state: Res<State<EditorState>>,
) {
    if trigger.event().0 == **state {
        return;
    }
    info!(?state, "Changing state");
    commands.trigger(ClearCreatedComponentEv);
    commands.insert_resource(NextState::Pending(trigger.event().0));
}

pub trait IntoSystemConfigExt<T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>, Marker>:
    IntoScheduleConfigs<T, Marker>
{
    fn run_if_not_loading(self) -> ScheduleConfigs<T> {
        self.into_configs()
            .run_if(not(in_state(EditorState::Loading)))
    }
}

impl<X, T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>, Marker>
    IntoSystemConfigExt<T, Marker> for X
where
    X: IntoScheduleConfigs<T, Marker>,
{
}
