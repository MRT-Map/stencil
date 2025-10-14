use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::{
    dirs_paths::{data_dir, data_path},
    impl_load_save,
    settings::Settings,
    settings_field,
};

settings_field!(MiscSettings, notif_duration_is_default, notif_duration, u64);

#[derive(Deserialize, Serialize, Clone, PartialEq)]
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

impl_load_save!(toml MiscSettings, data_dir("settings").join("misc.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#misc_settingstoml");

impl Settings for MiscSettings {
    fn ui_inner(&mut self, ui: &mut Ui) {
        ui.add(
            egui::Slider::new(&mut self.notif_duration, 0..=10)
                .suffix("s")
                .text("Notification duration"),
        );
        ui.label("Time before success and info notifications expire. Set to 0 to disable expiry");
    }
}
