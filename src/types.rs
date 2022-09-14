pub mod pla;
pub mod skin;
pub mod tile_coord;
pub mod zoom;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

use crate::{
    editor::shadow::SelectShadow,
    pla::{ComponentCoords, CreatedComponent, EditorComponent, SelectedComponent},
    Skin,
};

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
pub enum Label {
    MenuUi,
    ComponentPanelUi,
    ToolbarUi,
    Controls,
    Cleanup,
}
impl SystemLabel for Label {
    fn as_str(&self) -> &'static str {
        self.into()
    }
}

pub type DeselectQuery<'world, 'state, 'a> = (
    Query<
        'world,
        'state,
        (&'a EditorComponent, &'a ComponentCoords, Entity),
        With<SelectedComponent>,
    >,
    Query<'world, 'state, Entity, With<SelectShadow>>,
    Res<'world, Skin>,
);
pub type SelectQuery<'world, 'state, 'a, F = ()> = ParamSet<
    'world,
    'state,
    (
        DeselectQuery<'world, 'state, 'a>,
        (
            Query<'world, 'state, (&'a EditorComponent, &'a mut ComponentCoords, Entity), F>,
            Res<'world, Skin>,
        ),
    ),
>;
pub type CreatedQuery<'world, 'state, 'a> = Query<
    'world,
    'state,
    (&'a EditorComponent, &'a mut ComponentCoords, Entity),
    With<CreatedComponent>,
>;
