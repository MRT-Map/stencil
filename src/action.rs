use std::{any::Any, sync::Arc};

use bevy::prelude::*;

#[derive(Event, Clone)]
pub struct Action(Arc<dyn Any + Send + Sync>);
impl Action {
    pub fn new<T: Any + Send + Sync>(v: T) -> Self {
        Self(Arc::new(v))
    }
    #[must_use]
    pub fn downcast_ref<R: Any>(&self) -> Option<&R> {
        self.0.as_ref().downcast_ref()
    }
}
