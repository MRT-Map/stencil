pub mod settings;
pub mod settings_editor;

use bevy::prelude::*;

use crate::misc_config::{settings::INIT_MISC_SETTINGS, settings_editor::misc_settings_asy};

pub struct MiscSettingsPlugin;

impl Plugin for MiscSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(INIT_MISC_SETTINGS.to_owned())
            .add_systems(Update, misc_settings_asy);
    }
}
