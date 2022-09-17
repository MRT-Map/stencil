use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::entity::ShapeBundle;

use crate::types::{pla::EditorCoords, skin::Skin};
use crate::types::pla::PlaComponent;

#[derive(Bundle)]
pub struct ComponentBundle {
    pub data: PlaComponent<EditorCoords>,

    #[bundle]
    pub shape: ShapeBundle,
    #[bundle]
    pub pickable: PickableBundle,
}

impl ComponentBundle {
    pub fn new(data: PlaComponent<EditorCoords>) -> Self {
        Self {
            data,
            shape: ShapeBundle::default(),
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
