use bevy::prelude::*;

pub mod changelog;
pub mod info;
pub mod licenses;
pub mod quit;

#[derive(Clone)]
pub enum InfoWindowsAct {
    Changelog,
    Info,
    Licenses,
    Quit(bool),
}

pub struct InfoWindowsPlugin;

impl Plugin for InfoWindowsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(quit::quit_asy)
            .add_system(info::info_asy)
            .add_system(changelog::changelog_asy)
            .add_system(licenses::licenses_asy);
    }
}
