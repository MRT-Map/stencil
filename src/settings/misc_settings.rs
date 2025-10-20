use std::{any::Any, sync::atomic::Ordering};

use serde::{Deserialize, Serialize};

use crate::{
    file::data_dir,
    impl_load_save,
    settings::{Settings, settings_ui_field},
    settings_field,
    ui::notif::NOTIF_DURATION,
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

impl_load_save!(toml MiscSettings, data_dir("settings").join("misc.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#misc_settingstoml");

impl Settings for MiscSettings {
    fn ui_inner(&mut self, ui: &mut egui::Ui, _tab_state: &mut dyn Any) {
        let default = Self::default();

        let mut changed = false;
        settings_ui_field(
            ui,
            &mut self.notif_duration,
            default.notif_duration,
            Some("Time before success and info notifications expire. Set to 0 to disable expiry"),
            |ui, value| {
                changed = ui
                    .add(
                        egui::Slider::new(value, 0..=10)
                            .suffix("s")
                            .text("Notification duration"),
                    )
                    .changed();
            },
        );
        if changed {
            self.update_notif_duration();
        }
    }
}

impl MiscSettings {
    pub fn update_notif_duration(&self) {
        NOTIF_DURATION.store(self.notif_duration, Ordering::Relaxed);
    }
}
