use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

use crate::{
    editor::bundles::component::{CreatedComponent, SelectedComponent},
    types::{
        pla::{EditorCoords, PlaComponent},
        skin::Skin,
    },
};

pub mod pla;
pub mod skin;
pub mod tile_coord;
pub mod zoom;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ComponentType {
    #[serde(rename = "point")]
    Point,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "area")]
    Area,
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditorState {
    #[default]
    Loading,
    Idle,
    CreatingComponent(ComponentType),
    EditingNodes,
    MovingComponent,
    RotatingComponent,
    DeletingComponent,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, IntoStaticStr)]
#[non_exhaustive]
pub enum Label {
    Ui,
    Controls,
    Cleanup,
    Select,
    HighlightSelected,
    CreateComponent,
}
impl SystemLabel for Label {
    fn as_str(&self) -> &'static str {
        self.into()
    }
}

pub type DeselectQuery<'world, 'state, 'a> = (
    Query<'world, 'state, (&'a PlaComponent<EditorCoords>, Entity), With<SelectedComponent>>,
    Res<'world, Skin>,
);
pub type CreatedQuery<'world, 'state, 'a> =
    Query<'world, 'state, (&'a mut PlaComponent<EditorCoords>, Entity), With<CreatedComponent>>;
