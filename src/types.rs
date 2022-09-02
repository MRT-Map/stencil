pub mod pla;
pub mod skin;
pub mod zoom;
pub mod tile_coord;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ComponentType {
    #[serde(rename = "point")]
    Point,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "area")]
    Area,
}

#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditorState {
    #[default]
    Loading,
    Idle,
    CreatingComponent(ComponentType),
    EditingNodes,
    MovingComponent,
    RotatingComponent,
    DeletingComponent,
}

#[derive(Default)]
pub struct HoveringOverGui(pub bool);
