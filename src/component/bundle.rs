use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::component::{
    pla2::{EditorCoords, PlaComponent},
    skin::Skin,
};

#[derive(Bundle)]
pub struct ComponentBundle {
    pub pla: PlaComponent<EditorCoords>,
    pub shape: (Shape, Transform),
    pub pickable: (Pickable, RayCastBackfaces),
}
impl ComponentBundle {
    #[must_use]
    pub fn new(pla: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            shape: pla.get_shape(skin),
            pla,
            pickable: (Pickable::default(), RayCastBackfaces),
        }
    }
}
