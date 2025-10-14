use bevy::prelude::*;

use crate::component::{pla2::PlaComponent, skin::Skin};

pub mod pla2;
pub mod skin;

pub mod actions;
pub mod circle;
pub mod panels;
pub mod tools;

#[must_use]
pub fn make_component(pla: PlaComponent, skin: &Skin) -> impl Bundle {
    (
        pla.get_shape(skin),
        pla,
        Pickable::default(),
        RayCastBackfaces,
    )
}
