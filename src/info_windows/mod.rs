use bevy::prelude::*;

pub mod changelog;
pub mod info;
pub mod licenses;
pub mod manual;
pub mod quit;

#[derive(Clone, Copy, Event, PartialEq, Eq)]
pub enum InfoWindowsEv {
    Changelog,
    Info,
    Licenses,
    Manual,
    Quit(bool),
}

pub struct InfoWindowsPlugin;

impl Plugin for InfoWindowsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(quit::on_quit)
            // .add_systems(
            //     Update,
            //     info::on_info.run_if(resource_exists::<ImageAssets>),
            // )
            .add_observer(info::on_info)
            .add_observer(changelog::on_changelog)
            .add_observer(manual::on_manual)
            .add_observer(licenses::on_license);
    }
}
