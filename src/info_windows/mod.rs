use bevy::prelude::*;

pub mod changelog;
pub mod info;
pub mod licenses;
pub mod manual;
pub mod quit;

#[derive(Clone, Copy, Event, PartialEq, Eq)]
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
        app.observe(quit::on_quit)
            // .add_systems(
            //     Update,
            //     info::on_info.run_if(resource_exists::<ImageAssets>),
            // )
            .observe(info::on_info)
            .observe(changelog::on_changelog)
            .observe(manual::on_manual)
            .observe(licenses::on_license);
    }
}
