use bevy::prelude::*;

pub mod changelog;
pub mod info;
pub mod quit;

pub struct InfoWindowsPlugin;

impl Plugin for InfoWindowsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(quit::quit_msy)
            .add_system(info::info_msy)
            .add_system(changelog::changelog_msy);
    }
}
