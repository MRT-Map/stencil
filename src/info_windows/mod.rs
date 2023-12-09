use bevy::prelude::*;

use crate::init::load_assets::ImageAssets;

pub mod changelog;
pub mod info;
pub mod licenses;
pub mod manual;
pub mod quit;

#[derive(Clone)]
pub enum InfoWindowsAct {
    Changelog,
    Info,
    Licenses,
    Manual,
    Quit(bool),
}

pub struct InfoWindowsPlugin;

impl Plugin for InfoWindowsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, quit::quit_asy)
            .add_systems(
                Update,
                info::info_asy.run_if(resource_exists::<ImageAssets>()),
            )
            .add_systems(Update, changelog::changelog_asy)
            .add_systems(Update, manual::manual_asy)
            .add_systems(Update, licenses::licenses_asy);
    }
}
