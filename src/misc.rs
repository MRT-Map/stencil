use std::{any::Any, fmt::Display};

use crate::pla2::component::ComponentType;

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditorState {
    #[default]
    Loading,
    Idle,
    CreatingComponent(ComponentType),
    EditingNodes,
    DeletingComponent,
}

pub struct Action<P: Send + Sync + ?Sized = dyn Any + Send + Sync> {
    pub id: String,
    pub payload: Box<P>,
}
impl Action {
    pub fn new(id: impl Display) -> Self {
        Self {
            id: id.to_string(),
            payload: Box::new(()),
        }
    }
}
