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
    pub const fn is_editing(self) -> bool {
        matches!(
            self,
            Self::CreatePoint | Self::CreateLine | Self::CreateArea
        )
    }
}
