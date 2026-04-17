use std::any::Any;

use egui::Ui;
use etcetera::AppStrategy;
use serde::{Deserialize, Serialize};

use crate::{file::FOLDERS, impl_load_save, settings::Settings};

#[expect(clippy::empty_structs_with_brackets)]
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct UiSettings {}

impl_load_save!(toml UiSettings, FOLDERS.in_config_dir("ui.toml"), "# Documentation is at https://github.com/MRT-Map/stencil3/wiki/Advanced-Topics#settings.uitoml");

impl Settings for UiSettings {
    fn ui_inner(&mut self, ui: &mut Ui, _tab_state: &mut dyn Any) {
        ui.ctx().clone().settings_ui(ui);
    }
}
