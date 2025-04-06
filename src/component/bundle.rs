use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::component::{
    pla2::{EditorCoords, PlaComponent},
    skin::Skin,
};

#[derive(Bundle)]
pub struct PointComponentBundle {
    pub data: PlaComponent<EditorCoords>,
    pub shape: ShapeBundle,
    pub fill: Fill,
    pub pickable: (RayCastPickable, RayCastBackfaces),
}
impl PointComponentBundle {
    #[must_use]
    pub fn new(data: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            shape: data.get_shape(skin),
            fill: data.get_fill(skin),
            data,
            pickable: (RayCastPickable, RayCastBackfaces),
        }
    }
}

#[derive(Bundle)]
pub struct LineComponentBundle {
    pub data: PlaComponent<EditorCoords>,
    pub shape: ShapeBundle,
    pub stroke: Stroke,
    pub pickable: (RayCastPickable, RayCastBackfaces),
}
impl LineComponentBundle {
    #[must_use]
    pub fn new(data: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            shape: data.get_shape(skin),
            stroke: data.get_stroke(skin),
            data,
            pickable: (RayCastPickable, RayCastBackfaces),
        }
    }
}

#[derive(Bundle)]
pub struct AreaComponentBundle {
    pub data: PlaComponent<EditorCoords>,
    pub shape: ShapeBundle,
    pub fill: Fill,
    pub stroke: Stroke,
    pub pickable: (RayCastPickable, RayCastBackfaces),
}
impl AreaComponentBundle {
    #[must_use]
    pub fn new(data: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            shape: data.get_shape(skin),
            fill: data.get_fill(skin),
            stroke: data.get_stroke(skin),
            data,
            pickable: (RayCastPickable, RayCastBackfaces),
        }
    }
}