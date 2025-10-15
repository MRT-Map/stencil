use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum EditorMode {
    #[default]
    Select,
    Nodes,
    CreatePoint,
    CreateLine,
    CreateArea,
}

impl EditorMode {
    // #[must_use]
    // pub const fn component_type(self) -> Option<ComponentType> {
    //     match self {
    //         Self::CreatingArea => Some(ComponentType::Area),
    //         Self::CreatingLine => Some(ComponentType::Line),
    //         Self::CreatingPoint => Some(ComponentType::Point),
    //         _ => None,
    //     }
    // }
}
