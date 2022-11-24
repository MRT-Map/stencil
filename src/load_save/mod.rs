use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

use bevy::prelude::*;

use crate::pla2::component::{MCCoords, PlaComponent};

pub mod load_ns;
pub mod save_ns;

#[derive(Clone)]
pub enum LoadSaveAct {
    Load,
    Load1(Option<BTreeSet<PathBuf>>),
    Load2(PathBuf, Arc<BTreeSet<String>>),
    Load3(Vec<PlaComponent<MCCoords>>),
    Save,
    Save1(Option<PathBuf>),
}

pub struct LoadSavePlugin;

impl Plugin for LoadSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(load_ns::load_ns_asy)
            .add_system(save_ns::save_ns_asy);
    }
}
