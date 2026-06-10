pub mod misc_settings;

use std::{any::Any, fmt::Display};

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App,
    map::settings::MapSettings,
    settings::misc_settings::MiscSettings,
    shortcut::settings::{ShortcutSettings, ShortcutsTabState},
    ui::{dock::DockWindow, settings::UiSettings},
    utils::load_save::LoadSave,
};

#[derive(Default)]
pub struct AppSettings {
    pub map: MapSettings,
    pub ui: UiSettings,
    pub shortcut: ShortcutSettings,
    pub misc: MiscSettings,
}

impl AppSettings {
    pub fn load_state() -> Self {
        Self {
            map: MapSettings::load(),
            ui: UiSettings::load(),
            shortcut: ShortcutSettings::load(),
            misc: MiscSettings::load(),
        }
    }
    pub fn save_state(&self) {
        self.misc.save();
        self.shortcut.save();
        self.map.save();
        self.ui.save();
    }
}

#[macro_export]
macro_rules! settings {
    ($(#[$attr2:meta])* $Ty:ident { $( $(#[$attr:meta])? $field:ident : $field_ty:ty = $default:expr,)* }) => { paste::paste! {
        $(#[$attr2])*
        #[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Debug)]
        #[serde(default)]
        pub struct $Ty {
            $($(#[$attr])? #[serde(skip_serializing_if = $Ty "Default::" $field)] pub $field: $field_ty),*
        }

        impl Default for $Ty {
            fn default() -> Self {
                Self {
                    $($field: $default),*
                }
            }
        }

        pub struct [<$Ty Default>];
        impl [<$Ty Default>] {
            $(
                #[expect(clippy::allow_attributes)]
                #[allow(clippy::float_cmp)]
                fn $field(v: &$field_ty) -> bool {
                    *v == <$Ty>::default().$field
                }
            )*
        }}
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

pub fn settings_ui_field<
    T: PartialEq + Display,
    D: Into<egui::WidgetText>,
    F: FnOnce(&mut egui::Ui, &mut T),
>(
    ui: &mut egui::Ui,
    value: &mut T,
    default: T,
    description: Option<D>,
    edit_ui: F,
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
pub fn settings_ui_field_no_display<
    T: PartialEq,
    TD: Display,
    D: Into<egui::WidgetText>,
    F: FnOnce(&mut egui::Ui, &mut T),
>(
    ui: &mut egui::Ui,
    value: &mut T,
    default: T,
    default_display: TD,
    description: Option<D>,
    edit_ui: F,
) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(*value != default, egui::Button::new("⟲"))
            .on_hover_text(format!("Default: {default_display}"))
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
    Ui,
    Shortcuts(ShortcutsTabState),
    Miscellaneous,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub struct SettingsWindow {
    tab: SettingsTab,
}

impl DockWindow for SettingsWindow {
    fn title(self) -> String {
        "Settings".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        egui::Panel::top("select_settings").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
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
                selectable_button!("UI", SettingsTab::Ui, SettingsTab::Ui);
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
        });

        egui::ScrollArea::vertical().show(ui, |ui| match &mut self.tab {
            SettingsTab::Map => {
                app.settings.map.ui(ui, &mut ());
            }
            SettingsTab::Ui => app.settings.ui.ui(ui, &mut ()),
            SettingsTab::Shortcuts(state) => {
                app.settings.shortcut.ui(ui, state);
            }
            SettingsTab::Miscellaneous => {
                app.settings.misc.ui(ui, &mut ());
            }
        });
    }
}
