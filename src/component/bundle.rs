use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy::render::primitives::Aabb;
use bevy_prototype_lyon::prelude::*;

use crate::component::{
    pla2::{EditorCoords, PlaComponent, Select},
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

pub trait EntityCommandsSelectExt {
    fn select_component(&mut self, skin: &Skin, data: &PlaComponent<EditorCoords>) -> &mut Self;
    fn component_display(&mut self, skin: &Skin, data: &PlaComponent<EditorCoords>) -> &mut Self;
}

impl EntityCommandsSelectExt for EntityCommands<'_> {
    fn select_component(&mut self, skin: &Skin, data: &PlaComponent<EditorCoords>) -> &mut Self {
        self.remove::<Aabb>();
        let ty = data.get_type(skin);
        let fill = data.get_fill(skin).select(ty).to_owned();
        if fill.color == Color::NONE {
            self.remove::<Fill>();
        } else {
            self.insert(fill);
        }
        let stroke = data.get_stroke(skin).select(ty).to_owned();
        if stroke.color == Color::NONE {
            self.remove::<Stroke>();
        } else {
            self.insert(stroke);
        }
        self.insert(data.get_shape(skin));
        self
    }
    fn component_display(&mut self, skin: &Skin, data: &PlaComponent<EditorCoords>) -> &mut Self {
        self.remove::<Aabb>();
        let fill = data.get_fill(skin);
        if fill.color == Color::NONE {
            self.remove::<Fill>();
        } else {
            self.insert(fill);
        }
        let stroke = data.get_stroke(skin);
        if stroke.color == Color::NONE {
            self.remove::<Stroke>();
        } else {
            self.insert(stroke);
        }
        self.insert(data.get_shape(skin));
        self
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct CreatedComponent;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedComponent;
