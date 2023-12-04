use bevy::{ecs::system::EntityCommands, math::cubic_splines::Point, prelude::*};
use bevy_mod_picking::{backends::raycast::RaycastPickable, prelude::*};
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*};

use crate::pla2::{
    component::{ComponentType, EditorCoords, PlaComponent, Select},
    skin::Skin,
};

pub trait ComponentBundle {
    fn data(&self) -> &PlaComponent<EditorCoords>;
    fn data_mut(&mut self) -> &mut PlaComponent<EditorCoords>;
    fn update_display(&mut self, skin: &Skin) -> &mut Self;
    fn select(&mut self) -> &mut Self;
    fn deselect(&mut self, skin: &Skin) -> &mut Self;
}

#[derive(Bundle)]
pub struct PointComponentBundle {
    pub data: PlaComponent<EditorCoords>,
    pub pickable: PickableBundle,
    pub shape: ShapeBundle,
    pub fill: Fill,
}
impl PointComponentBundle {
    pub fn new(data: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        let mut s = Self {
            pickable: PickableBundle::default(),
            shape: data.get_shape(skin),
            fill: data.get_fill(skin),
            data,
        };
        s
    }
}
impl ComponentBundle for PointComponentBundle {
    fn data(&self) -> &PlaComponent<EditorCoords> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut PlaComponent<EditorCoords> {
        &mut self.data
    }
    fn update_display(&mut self, skin: &Skin) -> &mut Self {
        self.shape = self.data.get_shape(skin);
        self.fill = self.data.get_fill(skin);
        self
    }
    fn select(&mut self) -> &mut Self {
        self.fill.color = Color::YELLOW;
        self
    }
    fn deselect(&mut self, skin: &Skin) -> &mut Self {
        self.fill = self.data.get_fill(skin);
        self
    }
}

#[derive(Bundle)]
pub struct LineComponentBundle {
    pub data: PlaComponent<EditorCoords>,
    pub pickable: PickableBundle,
    pub shape: ShapeBundle,
    pub stroke: Stroke,
}
impl LineComponentBundle {
    pub fn new(data: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            pickable: PickableBundle::default(),
            shape: data.get_shape(skin),
            stroke: data.get_stroke(skin),
            data,
        }
    }
}
impl ComponentBundle for LineComponentBundle {
    fn data(&self) -> &PlaComponent<EditorCoords> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut PlaComponent<EditorCoords> {
        &mut self.data
    }
    fn update_display(&mut self, skin: &Skin) -> &mut Self {
        self.shape = self.data.get_shape(skin);
        self.stroke = self.data.get_stroke(skin);
        self
    }
    fn select(&mut self) -> &mut Self {
        self.stroke.color = *Color::YELLOW.clone().set_a(0.5);
        self
    }
    fn deselect(&mut self, skin: &Skin) -> &mut Self {
        self.stroke = self.data.get_stroke(skin);
        self
    }
}

#[derive(Bundle)]
pub struct AreaComponentBundle {
    pub data: PlaComponent<EditorCoords>,
    pub pickable: PickableBundle,
    pub shape: ShapeBundle,
    pub fill: Fill,
    pub stroke: Stroke,
}
impl AreaComponentBundle {
    pub fn new(data: PlaComponent<EditorCoords>, skin: &Skin) -> Self {
        Self {
            pickable: PickableBundle::default(),
            shape: data.get_shape(skin),
            fill: data.get_fill(skin),
            stroke: data.get_stroke(skin),
            data,
        }
    }
}
impl ComponentBundle for AreaComponentBundle {
    fn data(&self) -> &PlaComponent<EditorCoords> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut PlaComponent<EditorCoords> {
        &mut self.data
    }
    fn update_display(&mut self, skin: &Skin) -> &mut Self {
        self.shape = self.data.get_shape(skin);
        self.fill = self.data.get_fill(skin);
        self.stroke = self.data.get_stroke(skin);
        self
    }
    fn select(&mut self) -> &mut Self {
        self.stroke.color = *Color::YELLOW.clone().set_a(0.5);
        self.fill.color = Color::YELLOW;
        self
    }
    fn deselect(&mut self, skin: &Skin) -> &mut Self {
        self.stroke = self.data.get_stroke(skin);
        self.fill = self.data.get_fill(skin);
        self
    }
}

pub trait EntityCommandsSelectExt {
    fn select_component(&mut self, skin: &Skin, data: &PlaComponent<EditorCoords>) -> &mut Self;
    fn component_display(&mut self, skin: &Skin, data: &PlaComponent<EditorCoords>) -> &mut Self;
}

impl<'w, 's, 'a> EntityCommandsSelectExt for EntityCommands<'w, 's, 'a> {
    fn select_component(&mut self, skin: &Skin, data: &PlaComponent<EditorCoords>) -> &mut Self {
        let ty = data.get_type(&skin).unwrap();
        let fill = data.get_fill(&skin).select(ty).to_owned();
        if fill.color != Color::NONE {
            self.insert(fill);
        }
        let stroke = data.get_stroke(&skin).select(ty).to_owned();
        if stroke.color != Color::NONE {
            self.insert(stroke);
        }
        self.insert(data.get_shape(skin));
        self
    }
    fn component_display(&mut self, skin: &Skin, data: &PlaComponent<EditorCoords>) -> &mut Self {
        let fill = data.get_fill(&skin);
        if fill.color != Color::NONE {
            self.insert(fill);
        }
        let stroke = data.get_stroke(&skin);
        if stroke.color != Color::NONE {
            self.insert(stroke);
        }
        self.insert(data.get_shape(skin));
        self
    }
}

/*#[derive(Bundle)]
pub struct ComponentBundle {
    pub data: PlaComponent<EditorCoords>,

    pub shape: (ShapeBundle, Fill, Stroke),
    pub pickable: PickableBundle,
}

impl ComponentBundle {
    #[must_use]
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
}*/

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct CreatedComponent;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedComponent;
