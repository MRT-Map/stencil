pub mod pla;
pub mod skin;
pub mod tile_coord;
pub mod zoom;

use bevy::prelude::*;
use bevy_mouse_tracking_plugin::MousePos;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

use crate::{
    editor::bundles::component::{CreatedComponent, EditorComponent, SelectedComponent},
    types::{pla::ComponentCoords, skin::Skin},
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
    Query<
        'world,
        'state,
        (&'a EditorComponent, &'a ComponentCoords, Entity),
        With<SelectedComponent>,
    >,
    Res<'world, Skin>,
);
pub type SelectQuery<'world, 'state, 'a, F = ()> = ParamSet<
    'world,
    'state,
    (
        DeselectQuery<'world, 'state, 'a>,
        Query<'world, 'state, Entity, F>,
    ),
>;
pub type CreatedQuery<'world, 'state, 'a> = Query<
    'world,
    'state,
    (&'a EditorComponent, &'a mut ComponentCoords, Entity),
    With<CreatedComponent>,
>;
pub type DetectMouseMoveOnClick<'world, 'a> = (Local<'a, Option<MousePos>>, Res<'world, MousePos>);
pub trait DetectMouseMoveOnClickExt {
    fn handle_press(&mut self, buttons: &Res<Input<MouseButton>>);
    fn handle_release(&mut self) -> bool;
}
impl DetectMouseMoveOnClickExt for DetectMouseMoveOnClick<'_, '_> {
    fn handle_press(&mut self, buttons: &Res<Input<MouseButton>>) {
        if buttons.just_pressed(MouseButton::Left)
            || buttons.just_pressed(MouseButton::Right)
            || buttons.just_pressed(MouseButton::Middle)
        {
            *self.0 = Some(*self.1)
        }
    }
    fn handle_release(&mut self) -> bool {
        if let Some(prev) = *self.0 {
            *self.0 = None;
            let curr = *self.1;
            (*prev - *curr).length_squared() > 4.0
        } else {
            false
        }
    }
}
