use std::{collections::HashMap, path::PathBuf, time::Duration};

use bevy::prelude::*;
use events::ProjectEv;

use crate::{
    dirs_paths::cache_dir, misc_config::settings::MiscSettings, state::EditorState, ui::UiSchedule,
};

pub mod events;
pub mod project_editor;

#[derive(Resource, Clone)]
pub struct Namespaces {
    pub dir: PathBuf,
    pub visibilities: HashMap<String, bool>,
    pub prev_used: String,
}

impl Default for Namespaces {
    fn default() -> Self {
        Self {
            dir: cache_dir("scratchpad"),
            visibilities: {
                let mut h = HashMap::new();
                h.insert("_misc".into(), true);
                h
            },
            prev_used: "_misc".into(),
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
pub fn autosave_sy(
    mut commands: Commands,
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
        commands.trigger(ProjectEv::Save(true));
        commands.trigger(ProjectEv::Reload);
        *last_save = Some(time);
    }
}

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Namespaces>()
            .add_systems(Update, autosave_sy)
            .add_observer(events::on_project)
            .add_observer(project_editor::on_project_editor)
            .add_systems(UiSchedule, events::project_dialog)
            .add_systems(OnExit(EditorState::Loading), |mut commands: Commands| {
                commands.trigger(ProjectEv::Reload);
                commands.trigger(ProjectEv::Show {
                    ns: "_misc".into(),
                    history_invoked: true,
                    notif: false,
                });
            });
    }
}
