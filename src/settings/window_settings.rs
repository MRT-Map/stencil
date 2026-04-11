use std::any::Any;

use egui::Ui;
use etcetera::AppStrategy;
use serde::{Deserialize, Serialize};

use crate::{file::FOLDERS, impl_load_save, settings::Settings};

#[expect(clippy::empty_structs_with_brackets)]
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct WindowSettings {}

impl_load_save!(toml WindowSettings, FOLDERS.in_config_dir("window.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#settings.windowtoml");

impl Settings for WindowSettings {
    fn ui_inner(&mut self, ui: &mut Ui, _tab_state: &mut dyn Any) {
        let mut options = ui.memory(|m| m.options.clone());
        options.ui(ui);
        ui.memory_mut(|m| m.options = options);
    }
}
