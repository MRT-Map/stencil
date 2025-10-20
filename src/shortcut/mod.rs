use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App, info_windows::InfoWindowEv, project::component_editor::ComponentEditorWindow,
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
    PanMapUp,
    PanMapDown,
    PanMapLeft,
    PanMapRight,
    ZoomMapIn,
    ZoomMapOut,
}
impl ShortcutAction {
    pub const fn eventless(self) -> bool {
        matches!(
            self,
            Self::PanMapUp
                | Self::PanMapDown
                | Self::PanMapLeft
                | Self::PanMapRight
                | Self::ZoomMapIn
                | Self::ZoomMapOut
        )
    }
}

impl App {
    pub fn shortcuts(&mut self, ctx: &egui::Context) {
        for shortcut in self.shortcut_settings.shortcuts_ordered() {
            let action = self.shortcut_settings.keyboard_to_action(shortcut).unwrap();
            if action.eventless() {
                continue;
            }
            if !ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                continue;
            }
            info!(
                ?action,
                shortcut = ctx.format_shortcut(&shortcut),
                "Handling shortcut"
            );
            match action {
                ShortcutAction::Quit => self.push_event(InfoWindowEv::Quit {
                    confirm: cfg!(debug_assertions),
                }),
                ShortcutAction::SettingsWindow => {
                    self.open_dock_window(SettingsWindow::default());
                }
                ShortcutAction::ComponentEditorWindow => {
                    self.open_dock_window(ComponentEditorWindow);
                }
                ShortcutAction::NotifLogWindow => {
                    self.open_dock_window(NotifLogWindow);
                }
                _ => {}
            }
        }
    }
}
