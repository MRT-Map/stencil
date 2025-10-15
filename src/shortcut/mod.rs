use egui::KeyboardShortcut;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    App, component_editor::ComponentEditorWindow, info_windows::InfoWindowEv,
    settings::SettingsWindow, ui::notif::NotifLogWindow,
};

pub mod settings;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumCount,
    strum::VariantArray,
)]
pub enum ShortcutAction {
    Quit,
    SettingsWindow,
    ComponentEditorWindow,
    NotifLogWindow,
}

impl App {
    pub fn shortcuts(&mut self, ctx: &egui::Context) {
        for shortcut in self.shortcut_settings.shortcuts_ordered() {
            if !ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                continue;
            }
            match self.shortcut_settings.keyboard_to_action(shortcut).unwrap() {
                ShortcutAction::Quit => self
                    .events
                    .push_back(InfoWindowEv::Quit { confirm: false }.into()),
                ShortcutAction::SettingsWindow => {
                    self.open_dock_window(SettingsWindow::default());
                }
                ShortcutAction::ComponentEditorWindow => {
                    self.open_dock_window(ComponentEditorWindow);
                }
                ShortcutAction::NotifLogWindow => {
                    self.open_dock_window(NotifLogWindow);
                }
            }
        }
    }
}
