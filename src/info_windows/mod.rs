use bevy::prelude::*;

pub mod changelog;
pub mod info;
pub mod licenses;
pub mod quit;

pub struct InfoWindowsPlugin;

impl Plugin for InfoWindowsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(quit::quit_asy)
            .add_system(info::info_asy)
            .add_system(changelog::changelog_asy)
            .add_system(licenses::licenses_asy);
    }
}
