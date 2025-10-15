use std::any::Any;

use bimap::BiMap;
use eyre::OptionExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, VariantArray};
use tracing::info;

use crate::{
    dirs_paths::data_dir,
    event::Events,
    impl_load_save,
    settings::{Settings, misc_settings::MiscSettings},
    shortcut::ShortcutAction,
    ui::dock::DockWindows,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ShortcutSettings(BiMap<ShortcutAction, egui::KeyboardShortcut>);

impl_load_save!(toml ShortcutSettings, data_dir("settings").join("shortcut.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#shortcut_settingstoml");

impl Default for ShortcutSettings {
    fn default() -> Self {
        let mut map = BiMap::new();
        map.insert(
            ShortcutAction::Quit,
            egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape),
        );
        map.insert(
            ShortcutAction::SettingsWindow,
            egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Comma),
        );
        map.insert(
            ShortcutAction::ComponentEditorWindow,
            egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::ALT,
                egui::Key::C,
            ),
        );
        map.insert(
            ShortcutAction::NotifLogWindow,
            egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::ALT,
                egui::Key::N,
            ),
        );
        Self(map)
    }
}

impl ShortcutSettings {
    pub fn action_to_keyboard(&mut self, action: ShortcutAction) -> egui::KeyboardShortcut {
        if let Some(shortcut) = self.0.get_by_left(&action) {
            return *shortcut;
        };
        let shortcut = *Self::default().0.get_by_left(&action).unwrap();
        self.insert(action, shortcut);
        shortcut
    }
    pub fn keyboard_to_action(&self, keyboard: egui::KeyboardShortcut) -> Option<ShortcutAction> {
        self.0.get_by_right(&keyboard).map(|a| *a)
    }
    pub fn insert(
        &mut self,
        action: ShortcutAction,
        mut keyboard: egui::KeyboardShortcut,
    ) -> bimap::Overwritten<ShortcutAction, egui::KeyboardShortcut> {
        #[cfg(not(target_os = "macos"))]
        if keyboard.modifiers.ctrl {
            keyboard.modifiers.command = true;
            keyboard.modifiers.ctrl = false;
        }

        self.0.insert(action, keyboard)
    }
    pub fn shortcuts_ordered(&self) -> Vec<egui::KeyboardShortcut> {
        self.0
            .right_values()
            .sorted_by_cached_key(|a| {
                a.modifiers.alt as u8
                    + a.modifiers.command as u8
                    + a.modifiers.ctrl as u8
                    + a.modifiers.shift as u8
                    + a.modifiers.mac_cmd as u8
            })
            .rev()
            .copied()
            .collect()
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug, Default, Eq, PartialEq)]
pub enum ShortcutsTabState {
    #[default]
    None,
    WaitForShortcut(ShortcutAction),
    ChangeSuccess(ShortcutAction),
    ChangeFail {
        changing: ShortcutAction,
        taken_by: ShortcutAction,
    },
}
impl ShortcutsTabState {
    pub fn changed_shortcut(&self) -> Option<ShortcutAction> {
        match self {
            ShortcutsTabState::None => None,
            ShortcutsTabState::WaitForShortcut(action) => Some(*action),
            ShortcutsTabState::ChangeSuccess(action) => Some(*action),
            ShortcutsTabState::ChangeFail { changing, .. } => Some(*changing),
        }
    }
}

impl Settings for ShortcutSettings {
    fn ui_inner(&mut self, ui: &mut egui::Ui, tab_state: &mut dyn Any) {
        let tab_state = tab_state.downcast_mut::<ShortcutsTabState>().unwrap();

        egui_extras::TableBuilder::new(ui)
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::remainder())
            .column(egui_extras::Column::remainder())
            .column(egui_extras::Column::auto())
            .header(10.0, |mut header| {
                header.col(|_| ());
                header.col(|ui| {
                    ui.label("Action");
                });
                header.col(|ui| {
                    ui.label("Shortcut");
                });
                header.col(|_| ());
            })
            .body(|mut body| {
                let mut default = Self::default();
                body.rows(10.0, ShortcutAction::COUNT, |mut row| {
                    let action = ShortcutAction::VARIANTS[row.index()];
                    let default_keyboard = default.action_to_keyboard(action);

                    row.col(|ui| {
                        if ui
                            .add_enabled(
                                self.action_to_keyboard(action) != default_keyboard,
                                egui::Button::new("⟲"),
                            )
                            .on_hover_text(format!(
                                "Default: {}",
                                ui.ctx().format_shortcut(&default_keyboard)
                            ))
                            .clicked()
                        {
                            self.insert(action, default_keyboard);
                            *tab_state = ShortcutsTabState::None;
                        }
                    });
                    row.col(|ui| {
                        ui.label(format!("{action:?}").replace("ShortcutAction::", ""));
                    });
                    row.col(|ui| {
                        ui.label(ui.ctx().format_shortcut(&self.action_to_keyboard(action)));

                        if tab_state.changed_shortcut() != Some(action) {
                            return;
                        }
                        match tab_state {
                            ShortcutsTabState::WaitForShortcut(_) => {
                                ui.colored_label(egui::Color32::YELLOW, "Listening for shortcut..");
                            }
                            ShortcutsTabState::ChangeSuccess(_) => {
                                ui.colored_label(egui::Color32::GREEN, "Successfully changed");
                            }
                            ShortcutsTabState::ChangeFail { taken_by, .. } => {
                                ui.colored_label(
                                    egui::Color32::RED,
                                    format!("Already taken by {taken_by:?}"),
                                );
                            }
                            _ => {}
                        }
                    });
                    row.col(|ui| {
                        if ui.button("Change").clicked() {
                            *tab_state = ShortcutsTabState::WaitForShortcut(action);
                        }
                    });
                })
            });

        let ShortcutsTabState::WaitForShortcut(action) = *tab_state else {
            return;
        };
        let Some(key) = ui.ctx().input(|i| i.keys_down.iter().next().copied()) else {
            return;
        };
        let new_shortcut = egui::KeyboardShortcut::new(ui.ctx().input(|i| i.modifiers), key);
        assert!(ui.ctx().input_mut(|i| i.consume_shortcut(&new_shortcut)));

        if let Some(taken_by) = self.keyboard_to_action(new_shortcut)
            && taken_by != action
        {
            info!(changing=?action, ?taken_by, new_shortcut=ui.ctx().format_shortcut(&new_shortcut), "Shortcut already taken");
            *tab_state = ShortcutsTabState::ChangeFail {
                changing: action,
                taken_by,
            }
        } else {
            info!(changing=?action, new_shortcut=ui.ctx().format_shortcut(&new_shortcut), "Shortcut change succeeded");
            self.insert(action, new_shortcut);
            *tab_state = ShortcutsTabState::ChangeSuccess(action);
        }
    }
}
