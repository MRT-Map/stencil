mod events;
pub mod history_viewer;

use std::sync::{Arc, RwLock};

use bevy::prelude::*;

use crate::{
    component::pla2::{EditorCoords, PlaComponent},
    state::IntoSystemConfigExt,
};

#[derive(Clone, Debug)]
pub enum HistoryEntry<T = Entity> {
    Component {
        entity: T,
        before: Option<Box<PlaComponent<EditorCoords>>>,
        after: Option<Box<PlaComponent<EditorCoords>>>,
    },
    Namespace {
        namespace: String,
        visible: bool,
    },
}

pub enum HistoryAct {
    NewHistory(Vec<HistoryEntry>),
    Undo,
    Redo,
}
impl HistoryAct {
    #[must_use]
    pub fn one_history(history: HistoryEntry) -> Self {
        Self::NewHistory(vec![history])
    }
}

#[derive(Resource, Default, Debug)]
pub struct History {
    pub undo_stack: Vec<Vec<HistoryEntry<Arc<RwLock<Entity>>>>>,
    pub redo_stack: Vec<Vec<HistoryEntry<Arc<RwLock<Entity>>>>>,
}

pub struct HistoryPlugin;
impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<History>()
            .add_systems(Update, events::history_asy.run_if_not_loading());
    }
}
