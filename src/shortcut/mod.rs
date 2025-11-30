use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App,
    info_windows::quit::QuitPopup,
    mode::EditorMode,
    project::{
        component_editor::ComponentEditorWindow, history_viewer::HistoryViewerWindow,
        project_editor::ProjectEditorWindow,
    },
    settings::SettingsWindow,
    shortcut::settings::ShortcutSettings,
    ui::notif::NotifLogWindow,
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
    HistoryViewerWindow,
    NotifLogWindow,
    ProjectEditorWindow,
    PanMapUp,
    PanMapDown,
    PanMapLeft,
    PanMapRight,
    ZoomMapIn,
    ZoomMapOut,
    ResetMapView,
    OpenProject,
    ReloadProject,
    SaveProject,
    Undo,
    Redo,
    Delete,
    EditorModeSelect,
    EditorModeNodes,
    EditorModeCreatePoint,
    EditorModeCreateLine,
    EditorModeCreateArea,
    Copy,
    Cut,
    Paste,
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
        let mut eframe_workaround_used = false;
        for shortcut in self.shortcut_settings.shortcuts_ordered() {
            let action = self.shortcut_settings.shortcut_to_action(shortcut).unwrap();
            if action.eventless() {
                continue;
            }
            if !ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                if !eframe_workaround_used
                    && ctx.input_mut(|i| match shortcut {
                        egui::KeyboardShortcut {
                            modifiers,
                            logical_key: egui::Key::C,
                        } => {
                            i.modifiers.matches_logically(modifiers)
                                && i.events.iter().any(|e| matches!(e, egui::Event::Copy))
                        }
                        egui::KeyboardShortcut {
                            modifiers,
                            logical_key: egui::Key::X,
                        } => {
                            i.modifiers.matches_logically(modifiers)
                                && i.events.iter().any(|e| matches!(e, egui::Event::Cut))
                        }
                        egui::KeyboardShortcut {
                            modifiers,
                            logical_key: egui::Key::V,
                        } => {
                            i.modifiers.matches_logically(modifiers)
                                && i.events.iter().any(|e| matches!(e, egui::Event::Paste(_)))
                        }
                        _ => false,
                    })
                    && !ctx.wants_keyboard_input()
                {
                    eframe_workaround_used = true;
                } else {
                    continue;
                }
            }

            info!(
                ?action,
                shortcut = ctx.format_shortcut(&shortcut),
                "Handling shortcut"
            );
            match action {
                ShortcutAction::Quit => {
                    self.add_popup(QuitPopup);
                }
                ShortcutAction::SettingsWindow => {
                    self.open_dock_window(SettingsWindow::default());
                }
                ShortcutAction::ComponentEditorWindow => {
                    self.open_dock_window(ComponentEditorWindow);
                }
                ShortcutAction::HistoryViewerWindow => {
                    self.open_dock_window(HistoryViewerWindow);
                }
                ShortcutAction::NotifLogWindow => {
                    self.open_dock_window(NotifLogWindow);
                }
                ShortcutAction::ProjectEditorWindow => {
                    self.open_dock_window(ProjectEditorWindow);
                }
                ShortcutAction::ResetMapView => {
                    self.map_reset_view();
                }
                ShortcutAction::SaveProject => {
                    self.project.save_notif(&mut self.ui.notifs);
                }
                ShortcutAction::EditorModeSelect => {
                    self.mode = EditorMode::Select;
                }
                ShortcutAction::EditorModeNodes => {
                    self.mode = EditorMode::Nodes;
                }
                ShortcutAction::EditorModeCreatePoint => {
                    self.mode = EditorMode::CreatePoint;
                }
                ShortcutAction::EditorModeCreateLine => {
                    self.mode = EditorMode::CreateLine;
                }
                ShortcutAction::EditorModeCreateArea => {
                    self.mode = EditorMode::CreateArea;
                }
                ShortcutAction::Undo => {
                    self.history_undo(ctx);
                }
                ShortcutAction::Redo => {
                    self.history_redo(ctx);
                }
                ShortcutAction::Delete => self.delete_selected_components(ctx),
                ShortcutAction::Copy => self.copy_selected_components(ctx),
                ShortcutAction::Cut => self.cut_selected_components(ctx),
                ShortcutAction::Paste => self.paste_clipboard_components(ctx),
                _ => {}
            }
        }
    }
}

pub trait UiButtonWithShortcutExt {
    fn button_with_shortcut<'a>(
        &mut self,
        atoms: impl egui::IntoAtoms<'a>,
        shortcut: ShortcutAction,
        shortcut_settings: &mut ShortcutSettings,
    ) -> egui::Response;
}

impl UiButtonWithShortcutExt for egui::Ui {
    fn button_with_shortcut<'a>(
        &mut self,
        atoms: impl egui::IntoAtoms<'a>,
        shortcut: ShortcutAction,
        shortcut_settings: &mut ShortcutSettings,
    ) -> egui::Response {
        self.add(
            egui::Button::new(atoms)
                .shortcut_text(shortcut_settings.format_action(shortcut, self.ctx())),
        )
    }
}
