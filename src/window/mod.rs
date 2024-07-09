pub mod settings;
pub mod settings_editor;

use bevy::prelude::*;

use crate::window::{settings::INIT_WINDOW_SETTINGS, settings_editor::on_window_settings};

pub struct WindowSettingsPlugin;

impl Plugin for WindowSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(INIT_WINDOW_SETTINGS.to_owned())
            .observe(on_window_settings);
    }
}
