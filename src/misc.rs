use std::any::Any;

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
    pub id: &'static str,
    pub payload: Box<P>,
}
