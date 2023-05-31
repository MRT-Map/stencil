pub mod settings;

use bevy::prelude::*;

use crate::window_settings::settings::INIT_WINDOW_SETTINGS;

pub struct WindowSettingsPlugin;

impl Plugin for WindowSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(INIT_WINDOW_SETTINGS.to_owned());
    }
}
