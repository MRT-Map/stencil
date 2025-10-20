pub mod misc_settings;

use std::{any::Any, fmt::Display};

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App, load_save::LoadSave, shortcut::settings::ShortcutsTabState, ui::dock::DockWindow,
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
            format!("Settings can also be edited at: {}", Self::path().display()),
        );
        self.description(ui);
        ui.separator();
        self.ui_inner(ui, tab_state);
    }
}

pub fn settings_ui_field<T: PartialEq + Display>(
    ui: &mut egui::Ui,
    value: &mut T,
    default: T,
    description: Option<impl Into<egui::WidgetText>>,
    edit_ui: impl FnOnce(&mut egui::Ui, &mut T),
) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(*value != default, egui::Button::new("⟲"))
            .on_hover_text(format!("Default: {default}"))
            .clicked()
        {
            *value = default;
        }

        edit_ui(ui, value);
    });
    if let Some(description) = description {
        ui.label(description);
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
            macro_rules! selectable_button {
                ($label:literal, $new_val:expr, $match_:pat) => {
                    if ui
                        .add(egui::Button::selectable(
                            matches!(self.tab, $match_),
                            $label,
                        ))
                        .clicked()
                    {
                        info!(tab = $label, "Switching settings tab");
                        self.tab = $new_val;
                    }
                };
            }
            selectable_button!("Map", SettingsTab::Map, SettingsTab::Map);
            selectable_button!("Window", SettingsTab::Window, SettingsTab::Window);
            selectable_button!(
                "Shortcuts",
                SettingsTab::Shortcuts(ShortcutsTabState::default()),
                SettingsTab::Shortcuts(_)
            );
            selectable_button!(
                "Miscellaneous",
                SettingsTab::Miscellaneous,
                SettingsTab::Miscellaneous
            );
        });
        ui.separator();
        match &mut self.tab {
            SettingsTab::Map => {
                app.map_settings.ui(ui, &mut ());
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
