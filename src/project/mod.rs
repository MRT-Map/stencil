use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;

use crate::{
    misc::{cache_dir, Action},
    project::project_editor::ProjectAct,
    state::EditorState,
};

pub mod project_editor;

#[derive(Resource, Clone)]
pub struct Namespaces {
    pub folder: PathBuf,
    pub visibilities: HashMap<String, bool>,
}

impl Default for Namespaces {
    fn default() -> Self {
        Self {
            folder: cache_dir("scratchpad"),
            visibilities: {
                let mut h = HashMap::new();
                h.insert("misc".into(), true);
                h
            },
        }
    }
}

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Namespaces>()
            .add_systems(Update, project_editor::project_msy)
            .add_systems(
                OnExit(EditorState::Loading),
                |mut actions: EventWriter<Action>| {
                    actions.send(Action::new(ProjectAct::GetNamespaces));
                    actions.send(Action::new(ProjectAct::Show("misc".into())));
                },
            );
    }
}
