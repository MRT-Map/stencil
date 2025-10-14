pub mod miscellaneous;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{App, load_save::LoadSave, ui::dock::DockWindow};

#[macro_export]
macro_rules! settings_field {
    ($s:ty, $f:ident, $i:ident, $t:ty) => {
        #[expect(clippy::allow_attributes)]
        #[allow(clippy::float_cmp)]
        fn $f(v: &$t) -> bool {
            *v == <$s>::default().$i
        }
    };
}

pub trait Settings: LoadSave {
    fn description(&self, _ui: &mut egui::Ui) {}
    fn ui_inner(&mut self, ui: &mut egui::Ui);
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.colored_label(
            egui::Color32::YELLOW,
            format!(
                "Settings can also be edited at: {}",
                Self::path().to_string_lossy()
            ),
        );
        self.description(ui);
        ui.separator();
        self.ui_inner(ui);
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug, Default, Eq, PartialEq)]
enum SettingsTab {
    #[default]
    Map,
    Window,
    Shortcuts,
    Miscellaneous,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub struct SettingsWindow {
    tab: SettingsTab,
}

impl DockWindow for SettingsWindow {
    fn title(&self) -> String {
        "Settings".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, SettingsTab::Map, "Map");
            ui.selectable_value(&mut self.tab, SettingsTab::Window, "Window");
            ui.selectable_value(&mut self.tab, SettingsTab::Shortcuts, "Shortcuts");
            ui.selectable_value(&mut self.tab, SettingsTab::Miscellaneous, "Miscellaneous");
        });
        ui.separator();
        match self.tab {
            SettingsTab::Map => {
                ui.label("Map");
            }
            SettingsTab::Window => {
                ui.label("Window");
            }
            SettingsTab::Shortcuts => {
                ui.label("Shortcuts");
            }
            SettingsTab::Miscellaneous => {
                ui.label("Miscellaneous");
                app.misc_settings.ui(ui);
            }
        }
    }
}
