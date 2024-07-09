pub mod settings;
pub mod settings_editor;

use bevy::prelude::*;

use crate::misc_config::settings::INIT_MISC_SETTINGS;

pub struct MiscSettingsPlugin;

impl Plugin for MiscSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(INIT_MISC_SETTINGS.to_owned())
            .observe(settings_editor::on_misc_settings);
    }
}
