use std::any::Any;

use etcetera::AppStrategy;

use crate::{
    impl_load_save, settings,
    settings::{Settings, settings_ui_field},
    utils::file::FOLDERS,
};

settings! {
    #[derive(Eq)] MiscSettings {
        notif_duration: u64 = 2,
        autosave_duration_mins: u64 = 2,
    }
}
impl_load_save!(toml MiscSettings, FOLDERS.in_config_dir("misc.toml"), "# Documentation is at https://mrt-map.github.io/stencil3/doc/Misc-Settings.html");

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

        settings_ui_field(
            ui,
            &mut self.autosave_duration_mins,
            default.autosave_duration_mins,
            Some("Duration between autosaves. Set to 0 to disable autosaving"),
            |ui, value| {
                ui.add(
                    egui::Slider::new(value, 0..=60)
                        .suffix("min")
                        .text("Autosave duration"),
                );
            },
        );
    }
}
