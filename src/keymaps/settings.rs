use std::collections::HashMap;

use bevy::prelude::{KeyCode, Resource};
use itertools::Either;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    component_actions::undo_redo::UndoRedoAct,
    info_windows::InfoWindowsAct,
    keymaps::{
        key_list::KEY_LIST,
        settings_editor::{OpenKeymapSettingsAct, KEYMAP_MENU},
    },
    misc::{data_path, Action},
    project::project_editor::ProjectAct,
    state::{ChangeStateAct, EditorState},
    ui::tilemap::{settings::TileSettings, settings_editor::TileSettingsAct},
    window::settings_editor::OpenWindowSettingsAct,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeymapAction {
    ChangeState(EditorState),
    Undo,
    Redo,
    SelectProjectFolder,
    SaveProject,
    TileSettings,
    WindowSettings,
    KeymapSettings,
    Quit,
}

impl KeymapAction {
    #[must_use]
    pub fn action(self) -> Action {
        match self {
            Self::ChangeState(state) => Action::new(ChangeStateAct(state)),
            Self::Undo => Action::new(UndoRedoAct::Undo),
            Self::Redo => Action::new(UndoRedoAct::Redo),
            Self::SelectProjectFolder => Action::new(ProjectAct::SelectFolder),
            Self::SaveProject => Action::new(ProjectAct::Save),
            Self::TileSettings => Action::new(TileSettingsAct::Open),
            Self::WindowSettings => Action::new(OpenWindowSettingsAct),
            Self::KeymapSettings => Action::new(OpenKeymapSettingsAct),
            Self::Quit => Action::new(InfoWindowsAct::Quit(false)),
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KeymapSettings(pub HashMap<KeymapAction, KeyCode>);

impl Default for KeymapSettings {
    fn default() -> Self {
        Self(
            [
                (
                    KeymapAction::ChangeState(EditorState::Idle),
                    KeyCode::Digit1,
                ),
                (
                    KeymapAction::ChangeState(EditorState::EditingNodes),
                    KeyCode::Digit2,
                ),
                (
                    KeymapAction::ChangeState(EditorState::DeletingComponent),
                    KeyCode::Digit3,
                ),
                (
                    KeymapAction::ChangeState(EditorState::CreatingPoint),
                    KeyCode::Digit4,
                ),
                (
                    KeymapAction::ChangeState(EditorState::CreatingLine),
                    KeyCode::Digit5,
                ),
                (
                    KeymapAction::ChangeState(EditorState::CreatingArea),
                    KeyCode::Digit6,
                ),
                (KeymapAction::Undo, KeyCode::KeyU),
                (KeymapAction::Redo, KeyCode::KeyY),
                (KeymapAction::SelectProjectFolder, KeyCode::KeyL),
                (KeymapAction::SaveProject, KeyCode::KeyS),
                (KeymapAction::TileSettings, KeyCode::KeyT),
                (KeymapAction::WindowSettings, KeyCode::KeyW),
                (KeymapAction::KeymapSettings, KeyCode::KeyK),
                (KeymapAction::Quit, KeyCode::Escape),
            ]
            .into_iter()
            .collect(),
        )
    }
}

impl KeymapSettings {
    pub fn load() -> Result<Self, toml::de::Error> {
        match std::fs::read_to_string(data_path("keymap_settings.toml")) {
            Ok(str) => {
                info!("Found keymap settings file");
                toml::from_str(&str).map(|a| Self::from_serializable(&a))
            }
            Err(e) => {
                info!("Couldn't find or open keymap settings file: {e:?}");
                let s = Self::default();
                let _ = s.save();
                Ok(s)
            }
        }
    }
    pub fn save(&self) -> Result<(), Either<std::io::Error, toml::ser::Error>> {
        info!("Saving keymap settings file");
        let prefix_text = "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#keymap_settingstoml";
        let serialized = toml::to_string_pretty(&self.as_serializable()).map_err(Either::Right)?;

        std::fs::write(
            data_path("keymap_settings.toml"),
            format!("{prefix_text}\n\n{serialized}"),
        )
        .map_err(Either::Left)
    }

    #[must_use]
    pub fn as_serializable(&self) -> HashMap<&str, HashMap<String, String>> {
        let default = Self::default();
        KEYMAP_MENU
            .iter()
            .map(|(cat, menu)| {
                (
                    *cat,
                    menu.iter()
                        .map(|a| &a.0)
                        .filter(|action| default.0[action] != self.0[action])
                        .map(|action| (format!("{action:?}"), format!("{:?}", self.0[action])))
                        .collect(),
                )
            })
            .collect()
    }

    #[must_use]
    pub fn from_serializable(o: &HashMap<String, HashMap<String, String>>) -> Self {
        let mut s = Self::default();
        for menu in o.values() {
            for (action, key) in menu {
                let action = s.0.keys().find(|a| format!("{a:?}") == *action).unwrap();
                let key = KEY_LIST.iter().find(|a| format!("{a:?}") == *key).unwrap();
                s.0.insert(*action, *key);
            }
        }
        s
    }
}

pub static INIT_KEYMAP_SETTINGS: Lazy<KeymapSettings> =
    Lazy::new(|| KeymapSettings::load().unwrap());
