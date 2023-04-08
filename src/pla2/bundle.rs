use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*};

use crate::pla2::{
    component::{EditorCoords, PlaComponent},
    skin::Skin,
};

#[derive(Bundle)]
pub struct ComponentBundle {
    pub data: PlaComponent<EditorCoords>,

    #[bundle]
    pub shape: (ShapeBundle, Fill, Stroke),
    #[bundle]
    pub pickable: PickableBundle,
}

impl ComponentBundle {
    pub fn new(data: PlaComponent<EditorCoords>) -> Self {
        Self {
            data,
            shape: (
                ShapeBundle::default(),
                Fill::color(Color::NONE),
                Stroke::color(Color::NONE),
            ),
            pickable: PickableBundle::default(),
        }
    }
    pub fn update_shape(&mut self, skin: &Skin) {
        debug!(selected = false, "Updating shape of component");
        self.shape = self.data.get_shape(skin, false);
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct CreatedComponent;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedComponent;
