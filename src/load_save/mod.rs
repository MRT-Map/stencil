use bevy::prelude::*;

pub mod load_ns;
pub mod save_ns;

pub struct LoadSavePlugin;

impl Plugin for LoadSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(load_ns::load_ns_asy)
            .add_system(save_ns::save_ns_asy);
    }
}
