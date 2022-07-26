use crate::{ComponentType, Skin};
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
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
    pub tags: Vec<String>,
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
}
#[derive(Component)]
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
    pub fn update_shape(&mut self) {

    }
}
#[derive(Component)]
pub struct CreatedComponent;
#[derive(Component)]
pub struct SelectedComponent;
