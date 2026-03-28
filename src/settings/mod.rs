pub mod misc_settings;
pub mod window_settings;

use std::{any::Any, fmt::Display};

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App,
    load_save::LoadSave,
    map::settings::MapSettings,
    settings::{misc_settings::MiscSettings, window_settings::WindowSettings},
    shortcut::settings::{ShortcutSettings, ShortcutsTabState},
    ui::{dock::DockWindow, notif::NotifState},
};

#[derive(Default)]
pub struct AppSettings {
    pub map: MapSettings,
    pub window: WindowSettings,
    pub shortcut: ShortcutSettings,
    pub misc: MiscSettings,
}

impl AppSettings {
    pub fn load_state(notifs: &mut NotifState) -> Self {
        Self {
            map: MapSettings::load(notifs),
            window: WindowSettings::load(notifs),
            shortcut: ShortcutSettings::load(notifs),
            misc: {
                let s = MiscSettings::load(notifs);
                s.update_notif_duration();
                s
            },
        }
    }
    pub fn save_state(&self, notifs: &mut NotifState) {
        self.misc.save(notifs);
        self.shortcut.save(notifs);
        self.map.save(notifs);
        self.window.save(notifs);
    }
}

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
        });

        egui::ScrollArea::vertical().show(ui, |ui| match &mut self.tab {
            SettingsTab::Map => {
                app.settings.map.ui(ui, &mut ());
            }
            SettingsTab::Window => app.settings.window.ui(ui, &mut ()),
            SettingsTab::Shortcuts(state) => {
                app.settings.shortcut.ui(ui, state);
            }
            SettingsTab::Miscellaneous => {
                app.settings.misc.ui(ui, &mut ());
            }
        });
    }
}
