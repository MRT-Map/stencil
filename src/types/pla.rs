use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::{skin::Skin, ComponentType};

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
            ..default()
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

#[derive(Component, Clone)]
pub struct ComponentCoords(pub Vec<IVec2>);
