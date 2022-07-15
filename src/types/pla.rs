use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PlaComponent {
    id: String,
    displayname: String,
    description: String,
    tags: Vec<String>,
    layer: f64,
    #[serde(rename = "type")]
    type_: String,
    attributes: HashMap<String, String>,
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PlaNode {
    x: i32,
    y: i32,
    connections: Vec<String>,
}
