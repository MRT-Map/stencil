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
        app.add_system(quit::quit_asy)
            .add_system(info::info_asy.run_if(resource_exists::<ImageAssets>()))
            .add_system(changelog::changelog_asy)
            .add_system(manual::manual_asy)
            .add_system(licenses::licenses_asy);
    }
}
