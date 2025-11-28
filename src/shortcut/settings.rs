use std::any::Any;

use bimap::BiMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, VariantArray};
use tracing::info;

use crate::{file::data_dir, impl_load_save, settings::Settings, shortcut::ShortcutAction};

#[derive(Serialize, Deserialize, Debug)]
pub struct ShortcutSettings(BiMap<ShortcutAction, egui::KeyboardShortcut>);

impl_load_save!(toml ShortcutSettings, data_dir("settings").join("shortcut.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#shortcut_settingstoml");

impl Default for ShortcutSettings {
    fn default() -> Self {
        let mut map = BiMap::new();
        macro_rules! shortcut {
            ($action:ident => $($modifier:ident)|+ + $key:ident) => {
                map.insert(
                    ShortcutAction::$action,
                    egui::KeyboardShortcut::new($(egui::Modifiers::$modifier)|+, egui::Key::$key),
                );
            };
            ($action:ident -> $key:ident) => {
                shortcut!($action => NONE + $key);
            };
        }

        shortcut!(Quit -> Escape);
        shortcut!(SettingsWindow => COMMAND + Comma);
        shortcut!(ComponentEditorWindow => COMMAND | ALT + C);
        shortcut!(HistoryViewerWindow => COMMAND | ALT + H);
        shortcut!(NotifLogWindow => COMMAND | ALT + N);
        shortcut!(ProjectEditorWindow => COMMAND | ALT + P);
        shortcut!(ResetMapView => COMMAND + Backtick);
        shortcut!(PanMapUp -> ArrowUp);
        shortcut!(PanMapDown -> ArrowDown);
        shortcut!(PanMapLeft -> ArrowLeft);
        shortcut!(PanMapRight -> ArrowRight);
        shortcut!(ZoomMapIn -> Equals);
        shortcut!(ZoomMapOut -> Minus);
        shortcut!(OpenProject => COMMAND + O);
        shortcut!(ReloadProject => COMMAND + R);
        shortcut!(SaveProject => COMMAND + S);
        shortcut!(Undo => COMMAND + Z);
        shortcut!(Redo => COMMAND | SHIFT + Z);
        shortcut!(Delete -> Delete);
        shortcut!(EditorModeSelect => COMMAND + Num1);
        shortcut!(EditorModeNodes => COMMAND + Num2);
        shortcut!(EditorModeCreatePoint => COMMAND + Num3);
        shortcut!(EditorModeCreateLine => COMMAND + Num4);
        shortcut!(EditorModeCreateArea => COMMAND + Num5);
        shortcut!(Copy => COMMAND + C);
        shortcut!(Cut => COMMAND + X);
        shortcut!(Paste => COMMAND + V);
        Self(map)
    }
}

impl ShortcutSettings {
    pub fn action_to_shortcut(&mut self, action: ShortcutAction) -> egui::KeyboardShortcut {
        if let Some(shortcut) = self.0.get_by_left(&action) {
            return *shortcut;
        }

        let shortcut = *Self::default().0.get_by_left(&action).unwrap();
        self.insert(action, shortcut);
        shortcut
    }
    pub fn format_action(&mut self, action: ShortcutAction, ctx: &egui::Context) -> String {
        ctx.format_shortcut(&self.action_to_shortcut(action))
    }
    pub fn shortcut_to_action(&self, shortcut: egui::KeyboardShortcut) -> Option<ShortcutAction> {
        self.0.get_by_right(&shortcut).copied()
    }
    pub fn insert(
        &mut self,
        action: ShortcutAction,
        mut shortcut: egui::KeyboardShortcut,
    ) -> bimap::Overwritten<ShortcutAction, egui::KeyboardShortcut> {
        #[cfg(not(target_os = "macos"))]
        if shortcut.modifiers.ctrl {
            shortcut.modifiers.command = true;
            shortcut.modifiers.ctrl = false;
        }

        self.0.insert(action, shortcut)
    }
    pub fn shortcuts_ordered(&self) -> Vec<egui::KeyboardShortcut> {
        self.0
            .right_values()
            .sorted_by_cached_key(|a| {
                u8::from(a.modifiers.alt)
                    + u8::from(a.modifiers.command)
                    + u8::from(a.modifiers.ctrl)
                    + u8::from(a.modifiers.shift)
                    + u8::from(a.modifiers.mac_cmd)
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
    pub const fn changed_shortcut(self) -> Option<ShortcutAction> {
        match self {
            Self::None => None,
            Self::WaitForShortcut(action) | Self::ChangeSuccess(action) => Some(action),
            Self::ChangeFail { changing, .. } => Some(changing),
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
            .body(|body| {
                let mut default = Self::default();
                body.rows(10.0, ShortcutAction::COUNT, |mut row| {
                    let action = ShortcutAction::VARIANTS[row.index()];
                    let default_keyboard = default.action_to_shortcut(action);

                    row.col(|ui| {
                        if ui
                            .add_enabled(
                                self.action_to_shortcut(action) != default_keyboard,
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
                        ui.label(format!("{action}"));
                    });
                    row.col(|ui| {
                        ui.label(self.format_action(action, ui.ctx()));

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
                                    format!("Already taken by {taken_by}"),
                                );
                            }
                            ShortcutsTabState::None => {}
                        }
                    });
                    row.col(|ui| {
                        if ui.button("Change").clicked() {
                            *tab_state = ShortcutsTabState::WaitForShortcut(action);
                        }
                    });
                });
            });

        let ShortcutsTabState::WaitForShortcut(action) = *tab_state else {
            return;
        };
        let Some(key) = ui.ctx().input(|i| i.keys_down.iter().next().copied()) else {
            return;
        };
        let mut new_shortcut = egui::KeyboardShortcut::new(ui.ctx().input(|i| i.modifiers), key);
        #[cfg(not(target_os = "macos"))]
        if new_shortcut.modifiers.ctrl {
            new_shortcut.modifiers.command = true;
            new_shortcut.modifiers.ctrl = false;
        }

        assert!(ui.ctx().input_mut(|i| i.consume_shortcut(&new_shortcut)));

        if let Some(taken_by) = self.shortcut_to_action(new_shortcut)
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
