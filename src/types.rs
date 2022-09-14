pub mod pla;
pub mod skin;
pub mod zoom;
pub mod tile_coord;

use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use crate::editor::shadow::SelectShadow;
use crate::pla::{ComponentCoords, CreatedComponent, EditorComponent, SelectedComponent};
use crate::Skin;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ComponentType {
    #[serde(rename = "point")]
    Point,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "area")]
    Area,
}

#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Default)]
pub struct HoveringOverGui(pub bool);

pub type DeselectQuery<'world, 'state, 'a> = (
    Query<'world, 'state, (&'a EditorComponent, &'a ComponentCoords, Entity), With<SelectedComponent>>,
    Query<'world, 'state, Entity, With<SelectShadow>>,
    Res<'world, Skin>,
);
pub type SelectQuery<'world, 'state, 'a, F = ()> = ParamSet<'world, 'state, (
    DeselectQuery<'world, 'state, 'a>,
    (
        Query<'world, 'state, (&'a EditorComponent, &'a mut ComponentCoords, Entity), F>,
        Res<'world, Skin>
    )
)>;
pub type CreatedQuery<'world, 'state, 'a> = Query<'world, 'state,
    (&'a EditorComponent, &'a mut ComponentCoords, Entity),
    With<CreatedComponent>,
>;