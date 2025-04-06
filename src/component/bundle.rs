use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::component::{
    pla2::{EditorCoords, PlaComponent},
    skin::Skin,
};

#[derive(Bundle)]
pub struct PointComponentBundle {
    pub pla: PlaComponent<EditorCoords>,
    pub shape: ShapeBundle,
    pub fill: Fill,
    pub pickable: (RayCastPickable, RayCastBackfaces),
}
impl PointComponentBundle {
    #[must_use]
    pub fn new(pla: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            shape: pla.get_shape(skin),
            fill: pla.get_fill(skin),
            pla,
            pickable: (RayCastPickable, RayCastBackfaces),
        }
    }
}

#[derive(Bundle)]
pub struct LineComponentBundle {
    pub pla: PlaComponent<EditorCoords>,
    pub shape: ShapeBundle,
    pub stroke: Stroke,
    pub pickable: (RayCastPickable, RayCastBackfaces),
}
impl LineComponentBundle {
    #[must_use]
    pub fn new(pla: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            shape: pla.get_shape(skin),
            stroke: pla.get_stroke(skin),
            pla,
            pickable: (RayCastPickable, RayCastBackfaces),
        }
    }
}

#[derive(Bundle)]
pub struct AreaComponentBundle {
    pub pla: PlaComponent<EditorCoords>,
    pub shape: ShapeBundle,
    pub fill: Fill,
    pub stroke: Stroke,
    pub pickable: (RayCastPickable, RayCastBackfaces),
}
impl AreaComponentBundle {
    #[must_use]
    pub fn new(pla: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            shape: pla.get_shape(skin),
            fill: pla.get_fill(skin),
            stroke: pla.get_stroke(skin),
            pla,
            pickable: (RayCastPickable, RayCastBackfaces),
        }
    }
}