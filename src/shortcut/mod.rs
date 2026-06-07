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
    utils::coord::nn,
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
    Escape,
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
    SaveProjectAs,
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
    SelectAll,
}

impl App {
    #[tracing::instrument(skip_all)]
    pub fn shortcuts(&mut self, ctx: &egui::Context) {
        let mut eframe_workaround_used = false;
        for shortcut in self.settings.shortcut.shortcuts_ordered() {
            let action = self.settings.shortcut.shortcut_to_action(shortcut).unwrap();
            if ctx.egui_wants_keyboard_input() {
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
                {
                    eframe_workaround_used = true;
                } else {
                    continue;
                }
            }

            self.run_action(ctx, action, Some(shortcut));
        }
    }
    pub fn run_action(
        &mut self,
        ctx: &egui::Context,
        action: ShortcutAction,
        shortcut: Option<egui::KeyboardShortcut>,
    ) {
        info!(
            ?action,
            shortcut = shortcut.map(|shortcut| ctx.format_shortcut(&shortcut)),
            "Running action"
        );
        match action {
            ShortcutAction::Escape => match self.mode {
                EditorMode::Select => self.add_popup(QuitPopup),
                EditorMode::Nodes | EditorMode::CreatePoint => self.mode = EditorMode::Select,
                EditorMode::CreateLine | EditorMode::CreateArea => {
                    if self.ui.map.created_nodes.len() <= 1 {
                        self.mode = EditorMode::Select;
                    } else {
                        self.ui.map.created_nodes.clear();
                    }
                }
            },
            ShortcutAction::SettingsWindow => {
                self.ui.dock_layout.open_window(SettingsWindow::default());
            }
            ShortcutAction::ComponentEditorWindow => {
                self.ui.dock_layout.open_window(ComponentEditorWindow);
            }
            ShortcutAction::HistoryViewerWindow => {
                self.ui.dock_layout.open_window(HistoryViewerWindow);
            }
            ShortcutAction::NotifLogWindow => self.ui.dock_layout.open_window(NotifLogWindow),
            ShortcutAction::ProjectEditorWindow => {
                self.ui.dock_layout.open_window(ProjectEditorWindow);
            }
            ShortcutAction::ResetMapView => self.map_reset_view(),
            ShortcutAction::OpenProject => self.open_project(),
            ShortcutAction::ReloadProject => self.reload_project(),
            ShortcutAction::SaveProject => self.save_project(),
            ShortcutAction::SaveProjectAs => self.save_project_as(),
            ShortcutAction::EditorModeSelect => self.mode = EditorMode::Select,
            ShortcutAction::EditorModeNodes => self.mode = EditorMode::Nodes,
            ShortcutAction::EditorModeCreatePoint => self.mode = EditorMode::CreatePoint,
            ShortcutAction::EditorModeCreateLine => self.mode = EditorMode::CreateLine,
            ShortcutAction::EditorModeCreateArea => self.mode = EditorMode::CreateArea,
            ShortcutAction::Undo => self.history_undo(),
            ShortcutAction::Redo => self.history_redo(),
            ShortcutAction::Delete => self.delete_selected_components(),
            ShortcutAction::Copy => self.copy_selected_components(),
            ShortcutAction::Cut => self.cut_selected_components(),
            ShortcutAction::Paste => self.paste_clipboard_components(),
            ShortcutAction::SelectAll => self.select_all(),
            ShortcutAction::PanMapUp => self.ui.map.shortcut_pan_delta.y -= 1.0,
            ShortcutAction::PanMapDown => self.ui.map.shortcut_pan_delta.y += 1.0,
            ShortcutAction::PanMapLeft => self.ui.map.shortcut_pan_delta.x -= 1.0,
            ShortcutAction::PanMapRight => self.ui.map.shortcut_pan_delta.x += 1.0,
            ShortcutAction::ZoomMapIn => self.ui.map.shortcut_zoom_delta += nn(1.0),
            ShortcutAction::ZoomMapOut => self.ui.map.shortcut_zoom_delta -= nn(1.0),
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
            egui::Button::new(atoms).shortcut_text(shortcut_settings.format_action(shortcut, self)),
        )
    }
}
