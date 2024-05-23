use bevy::prelude::*;
use bevy_egui::egui;
use once_cell::sync::Lazy;

use crate::{
    action::Action,
    dirs_paths::data_path,
    keymaps::{
        key_list::KEY_LIST,
        settings::{KeymapAction, KeymapSettings},
    },
    state::EditorState,
    ui::panel::dock::{DockWindow, PanelDockState, PanelParams, TabViewer},
    window::settings::WindowSettings,
};

pub struct OpenKeymapSettingsAct;

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
                        ui.style_mut().wrap = Some(false);
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

pub fn keymap_settings_asy(mut actions: EventReader<Action>, mut state: ResMut<PanelDockState>) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(OpenKeymapSettingsAct))
            && !state
                .state
                .iter_all_tabs()
                .any(|(_, a)| a.title() == KeymapSettingsEditor.title())
        {
            state.state.add_window(vec![KeymapSettingsEditor.into()]);
        }
    }
}

pub static KEYMAP_MENU: Lazy<[(&str, Vec<(KeymapAction, &str)>); 3]> = Lazy::new(|| {
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
            ]
            .into_iter()
            .collect(),
        ),
        (
            "Other",
            [
                (KeymapAction::Undo, "Undo"),
                (KeymapAction::Redo, "Redo"),
                (KeymapAction::SelectProjectFolder, "Select Project Folder"),
                (KeymapAction::SaveProject, "Save"),
                (KeymapAction::Quit, "Quit"),
            ]
            .into_iter()
            .collect(),
        ),
    ]
});
