pub mod misc_settings;

use std::{any::Any, fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    App, load_save::LoadSave, settings::misc_settings::MiscSettings,
    shortcut::settings::ShortcutsTabState, ui::dock::DockWindow,
};

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
    fn ui_inner(&mut self, ui: &mut egui::Ui, tab_state: &mut dyn Any);
    fn ui(&mut self, ui: &mut egui::Ui, tab_state: &mut dyn Any) {
        ui.colored_label(
            egui::Color32::YELLOW,
            format!(
                "Settings can also be edited at: {}",
                Self::path().to_string_lossy()
            ),
        );
        self.description(ui);
        ui.separator();
        self.ui_inner(ui, tab_state);
    }
    fn ui_field<T: PartialEq + Display>(
        &mut self,
        ui: &mut egui::Ui,
        get: impl Fn(Self) -> T,
        get_ref: impl Fn(&Self) -> &T,
        get_mut: impl Fn(&mut Self) -> &mut T,
        description: Option<impl Into<egui::WidgetText>>,
        edit_ui: impl FnOnce(&mut egui::Ui, &mut T),
    ) {
        ui.horizontal(|ui| {
            let default = get(Self::default());
            if ui
                .add_enabled(*get_ref(self) != default, egui::Button::new("⟲"))
                .on_hover_text(format!("Default: {default}"))
                .clicked()
            {
                *get_mut(self) = default
            };

            edit_ui(ui, get_mut(self))
        });
        if let Some(description) = description {
            ui.label(description);
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug, Default, Eq, PartialEq)]
enum SettingsTab {
    #[default]
    Map,
    Window,
    Shortcuts(ShortcutsTabState),
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
            if ui
                .add(egui::Button::selectable(
                    matches!(self.tab, SettingsTab::Map),
                    "Map",
                ))
                .clicked()
            {
                self.tab = SettingsTab::Map;
            }
            if ui
                .add(egui::Button::selectable(
                    matches!(self.tab, SettingsTab::Window),
                    "Window",
                ))
                .clicked()
            {
                self.tab = SettingsTab::Window;
            }
            if ui
                .add(egui::Button::selectable(
                    matches!(self.tab, SettingsTab::Shortcuts(_)),
                    "Shortcuts",
                ))
                .clicked()
            {
                self.tab = SettingsTab::Shortcuts(ShortcutsTabState::default());
            }
            if ui
                .add(egui::Button::selectable(
                    matches!(self.tab, SettingsTab::Miscellaneous),
                    "Miscellaneous",
                ))
                .clicked()
            {
                self.tab = SettingsTab::Miscellaneous;
            }
        });
        ui.separator();
        match &mut self.tab {
            SettingsTab::Map => {
                ui.label("Map");
            }
            SettingsTab::Window => {
                ui.label("Window");
            }
            SettingsTab::Shortcuts(state) => {
                app.shortcut_settings.ui(ui, state);
            }
            SettingsTab::Miscellaneous => {
                app.misc_settings.ui(ui, &mut ());
            }
        }
    }
}
