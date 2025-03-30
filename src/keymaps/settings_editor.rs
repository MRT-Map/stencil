use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    dirs_paths::data_path,
    keymaps::{
        key_list::KEY_LIST,
        settings::{KeymapAction, KeymapSettings},
    },
    state::EditorState,
    ui::panel::dock::{window_action_handler, DockWindow, PanelDockState, PanelParams, TabViewer},
};

#[derive(Clone, Copy, Event)]
pub struct OpenKeymapSettingsEv;

#[derive(Clone, Copy)]
pub struct KeymapSettingsEditor;

impl DockWindow for KeymapSettingsEditor {
    fn title(self) -> String {
        "Keymap Settings".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams {
            keymap_settings, ..
        } = tab_viewer.params;
        let old_settings = keymap_settings.to_owned();

        if ui
            .add_enabled(
                **keymap_settings != KeymapSettings::default(),
                egui::Button::new("Reset"),
            )
            .clicked()
        {
            **keymap_settings = KeymapSettings::default();
        }
        ui.colored_label(
            egui::Color32::YELLOW,
            format!(
                "Keymap settings can also be edited at: {}",
                data_path("keymap_settings.toml").to_string_lossy()
            ),
        );
        ui.separator();

        let existing_keys = keymap_settings.0.values().copied().collect::<Vec<_>>();
        for (heading, menu) in &*KEYMAP_MENU {
            ui.heading(*heading);
            for (action, label) in menu {
                let key = keymap_settings.0.get_mut(action).unwrap();
                egui::ComboBox::from_label(*label)
                    .selected_text(format!("{key:?}"))
                    .width(25.0)
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        for list_key in &*KEY_LIST {
                            if existing_keys.contains(list_key) {
                                continue;
                            }
                            ui.selectable_value(key, *list_key, format!("{list_key:?}"));
                        }
                    });
            }
        }

        if old_settings != **keymap_settings {
            keymap_settings.save().unwrap();
        }
    }
}

pub fn on_keymap_settings(
    _trigger: Trigger<OpenKeymapSettingsEv>,
    mut state: ResMut<PanelDockState>,
) {
    window_action_handler(&mut state, KeymapSettingsEditor);
}

pub static KEYMAP_MENU: std::sync::LazyLock<[(&str, Vec<(KeymapAction, &str)>); 5]> =
    std::sync::LazyLock::new(|| {
        [
            (
                "State",
                [
                    (KeymapAction::ChangeState(EditorState::Idle), "Select"),
                    (
                        KeymapAction::ChangeState(EditorState::EditingNodes),
                        "Edit Nodes",
                    ),
                    (
                        KeymapAction::ChangeState(EditorState::DeletingComponent),
                        "Delete",
                    ),
                    (
                        KeymapAction::ChangeState(EditorState::CreatingPoint),
                        "Point",
                    ),
                    (KeymapAction::ChangeState(EditorState::CreatingLine), "Line"),
                    (KeymapAction::ChangeState(EditorState::CreatingArea), "Area"),
                ]
                .into_iter()
                .collect(),
            ),
            (
                "Settings",
                [
                    (KeymapAction::TileSettings, "Tile"),
                    (KeymapAction::WindowSettings, "Window"),
                    (KeymapAction::KeymapSettings, "Keymap"),
                    (KeymapAction::MiscSettings, "Misc"),
                    (KeymapAction::AllSettings, "All"),
                ]
                .into_iter()
                .collect(),
            ),
            (
                "Project",
                [
                    (KeymapAction::OpenProject, "Open Project"),
                    (KeymapAction::SaveProject, "Save Project"),
                    (KeymapAction::ReloadProject, "Reload Project"),
                ]
                .into_iter()
                .collect(),
            ),
            (
                "Windows",
                [
                    (KeymapAction::ComponentEditor, "Component Editor"),
                    (KeymapAction::Project, "Project"),
                    (KeymapAction::ComponentList, "Component List"),
                    (KeymapAction::History, "History"),
                    (KeymapAction::NotifLog, "Notification Log"),
                ]
                .into_iter()
                .collect(),
            ),
            (
                "Other",
                [
                    (KeymapAction::Undo, "Undo"),
                    (KeymapAction::Redo, "Redo"),
                    (KeymapAction::Quit, "Quit"),
                ]
                .into_iter()
                .collect(),
            ),
        ]
    });
