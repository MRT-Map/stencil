use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;
use events::ProjectAct;

use crate::{action::Action, dirs_paths::cache_dir, state::EditorState};

pub mod events;
pub mod project_editor;

#[derive(Resource, Clone)]
pub struct Namespaces {
    pub folder: PathBuf,
    pub visibilities: HashMap<String, bool>,
    pub prev_used: String,
}

impl Default for Namespaces {
    fn default() -> Self {
        Self {
            folder: cache_dir("scratchpad"),
            visibilities: {
                let mut h = HashMap::new();
                h.insert("_misc".into(), true);
                h
            },
            prev_used: "_misc".into(),
        }
    }
}

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Namespaces>()
            .add_systems(Update, events::project_msy)
            .add_systems(
                OnExit(EditorState::Loading),
                |mut actions: EventWriter<Action>| {
                    actions.send(Action::new(ProjectAct::GetNamespaces));
                    actions.send(Action::new(ProjectAct::Show {
                        ns: "_misc".into(),
                        history_invoked: true,
                    }));
                },
            );
    }
}
