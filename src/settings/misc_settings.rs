use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::{file::data_dir, impl_load_save, settings::Settings, settings_field};

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

impl_load_save!(toml MiscSettings, data_dir("settings").join("misc.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#misc_settingstoml");

impl Settings for MiscSettings {
    fn ui_inner(&mut self, ui: &mut egui::Ui, _tab_state: &mut dyn Any) {
        self.ui_field(
            ui,
            |a| a.notif_duration,
            |a| &a.notif_duration,
            |a| &mut a.notif_duration,
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
