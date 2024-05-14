use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;

pub mod project_editor;

#[derive(Resource, Default, Clone)]
pub struct Namespaces {
    pub folder: PathBuf,
    pub visibilities: HashMap<String, bool>,
}

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Namespaces>()
            .add_systems(Update, project_editor::project_msy);
    }
}
