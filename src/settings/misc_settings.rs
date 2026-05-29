use std::any::Any;

use etcetera::AppStrategy;
use serde::{Deserialize, Serialize};

use crate::{
    impl_load_save,
    settings::{Settings, settings_ui_field},
    settings_field,
    utils::file::FOLDERS,
};

settings_field!(MiscSettings, notif_duration_is_default, notif_duration, u64);

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(default)]
pub struct MiscSettings {
    #[serde(skip_serializing_if = "notif_duration_is_default")]
    pub notif_duration: u64,
}

impl Default for MiscSettings {
    fn default() -> Self {
        Self { notif_duration: 2 }
    }
}

impl_load_save!(toml MiscSettings, FOLDERS.in_config_dir("misc.toml"), "# Documentation is at https://github.com/MRT-Map/stencil3/wiki/Advanced-Topics#settings.misctoml");

impl Settings for MiscSettings {
    #[tracing::instrument(skip_all)]
    fn ui_inner(&mut self, ui: &mut egui::Ui, _tab_state: &mut dyn Any) {
        let default = Self::default();

        settings_ui_field(
            ui,
            &mut self.notif_duration,
            default.notif_duration,
            Some("Time before success and info notifications expire. Set to 0 to disable expiry"),
            |ui, value| {
                ui.add(
                    egui::Slider::new(value, 0..=10)
                        .suffix("s")
                        .text("Notification duration"),
                );
            },
        );
    }
}
