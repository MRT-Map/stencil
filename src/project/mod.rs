use std::{collections::HashMap, path::PathBuf, time::Duration};

use bevy::prelude::*;
use events::ProjectAct;

use crate::{
    action::Action, dirs_paths::cache_dir, misc_config::settings::MiscSettings, state::EditorState,
    ui::panel::status::Status,
};

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

#[allow(clippy::needless_pass_by_value)]
pub fn autosave_sy(
    mut actions: EventWriter<Action>,
    mut last_save: Local<Option<Duration>>,
    time: Res<Time<Real>>,
    misc_settings: Res<MiscSettings>,
) {
    if misc_settings.autosave_interval == 0 {
        return;
    }
    let Some(last_save_time) = &*last_save else {
        *last_save = Some(time.elapsed());
        return;
    };
    let time = time.elapsed();
    if time - last_save_time.to_owned() >= Duration::from_secs(misc_settings.autosave_interval) {
        actions.send(Action::new(ProjectAct::Save(true)));
        actions.send(Action::new(ProjectAct::GetNamespaces));
        *last_save = Some(time);
    }
}

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Namespaces>()
            .add_systems(Update, (events::project_asy, autosave_sy))
            .add_systems(
                OnExit(EditorState::Loading),
                |mut actions: EventWriter<Action>| {
                    actions.send(Action::new(ProjectAct::GetNamespaces));
                    actions.send(Action::new(ProjectAct::Show {
                        ns: "_misc".into(),
                        history_invoked: true,
                        notif: false,
                    }));
                },
            );
    }
}
