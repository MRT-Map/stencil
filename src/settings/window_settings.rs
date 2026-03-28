use std::any::Any;

use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::{file::data_dir, impl_load_save, settings::Settings, settings_field};

settings_field!(
    WindowSettings,
    light_mode_style_is_default,
    light_mode_style,
    egui::Style
);
settings_field!(
    WindowSettings,
    dark_mode_style_is_default,
    dark_mode_style,
    egui::Style
);

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct WindowSettings {
    #[serde(skip_serializing_if = "light_mode_style_is_default")]
    pub light_mode_style: egui::Style,
    #[serde(skip_serializing_if = "dark_mode_style_is_default")]
    pub dark_mode_style: egui::Style,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            light_mode_style: egui::Style {
                visuals: egui::Visuals::light(),
                ..egui::Style::default()
            },
            dark_mode_style: egui::Style::default(),
        }
    }
}

impl_load_save!(toml WindowSettings, data_dir("settings").join("window.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#settings.windowtoml");

impl Settings for WindowSettings {
    fn ui_inner(&mut self, ui: &mut Ui, _tab_state: &mut dyn Any) {
        ui.horizontal(|ui| {
            egui::widgets::global_theme_preference_buttons(ui);
            ui.label("Theme")
        });

        let style = match ui.theme() {
            egui::Theme::Dark => &mut self.dark_mode_style,
            egui::Theme::Light => &mut self.light_mode_style,
        };
        let old_style = style.clone();
        style.ui(ui);
        ui.vertical_centered(|ui| {
            egui::widgets::reset_button_with(
                ui,
                style,
                "(to light mode)",
                Self::default().light_mode_style,
            );
        });
        if (*style) != old_style {
            ui.set_style_of(ui.theme(), style.clone());
        }
    }
}
