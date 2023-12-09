pub mod settings;
pub mod window;

use bevy::prelude::*;

use crate::window_settings::{settings::INIT_WINDOW_SETTINGS, window::window_settings_msy};

pub struct WindowSettingsPlugin;

impl Plugin for WindowSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(INIT_WINDOW_SETTINGS.to_owned())
            .add_systems(Update, window_settings_msy);
    }
}
