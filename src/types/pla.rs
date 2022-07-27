use crate::{ComponentType, Skin};
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default, Component)]
pub struct PlaComponent {
    id: String,
    displayname: String,
    description: String,
    tags: Vec<String>,
    layer: f64,
    #[serde(rename = "type")]
    type_: String,
    nodes: Vec<String>,
    attributes: HashMap<String, String>,
}
impl PlaComponent {
    pub fn new(type_: ComponentType) -> Self {
        Self {
            type_: format!(
                "simple{}",
                match type_ {
                    ComponentType::Point => "Point",
                    ComponentType::Line => "Line",
                    ComponentType::Area => "Area",
                }
            ),
            ..Default::default()
        }
    }
    pub fn get_type(&self, skin: &Skin) -> Option<ComponentType> {
        Some(skin.types.get(self.type_.as_str())?.get_type())
    }
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PlaNode {
    x: i32,
    y: i32,
    connections: Vec<String>,
}

#[derive(Debug, Default, Component)]
pub struct EditorComponent {
    pub namespace: String,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub tags: String,
    pub layer: f64,
    pub type_: String,
    pub attributes: HashMap<String, String>,
}
impl EditorComponent {
    pub fn new(type_: ComponentType) -> Self {
        Self {
            type_: format!(
                "simple{}",
                match type_ {
                    ComponentType::Point => "Point",
                    ComponentType::Line => "Line",
                    ComponentType::Area => "Area",
                }
            ),
            ..Default::default()
        }
    }
    pub fn get_type(&self, skin: &Skin) -> Option<ComponentType> {
        Some(skin.types.get(self.type_.as_str())?.get_type())
    }
    pub fn get_shape(&self, coords: ComponentCoords, skin: &Skin) -> ShapeBundle {
        if self.get_type(skin) == Some(ComponentType::Point) {
            GeometryBuilder::build_as(
                &shapes::Rectangle {
                    extents: Vec2::new(10.0, 10.0),
                    origin: RectangleOrigin::Center,
                },
                DrawMode::Fill(FillMode::color(Color::CYAN)),
                Transform::from_xyz(coords.0[0].x as f32, coords.0[0].y as f32, 0.0),
            )
        } else {
            GeometryBuilder::build_as(
                &{
                    let mut pb = PathBuilder::new();
                    for coord in coords.0 {
                        pb.move_to(coord.as_vec2());
                    }
                    pb.build()
                },
                DrawMode::Stroke(StrokeMode::new(Color::CYAN, 8.0)),
                Transform::default()
            )
        }
    }
}
#[derive(Component, Clone)]
pub struct ComponentCoords(pub Vec<IVec2>);

#[derive(Bundle)]
pub struct ComponentBundle {
    pub data: EditorComponent,
    pub coords: ComponentCoords,

    #[bundle]
    pub shape: ShapeBundle,
}
impl ComponentBundle {
    pub fn new(data: EditorComponent, orig_coords: IVec2) -> Self {
        Self {
            data,
            coords: ComponentCoords(vec![orig_coords]),
            shape: ShapeBundle::default(),
        }
    }
    pub fn update_shape(&mut self, skin: &Skin) {
        self.shape = self.data.get_shape(self.coords.to_owned(), skin);
    }
}
#[derive(Component)]
pub struct CreatedComponent;
#[derive(Component)]
pub struct SelectedComponent;
