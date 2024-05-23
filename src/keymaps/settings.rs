use std::collections::HashMap;

use bevy::prelude::{KeyCode, Resource};
use eyre::OptionExt;
use itertools::Itertools;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    action::Action,
    component::panels::{
        component_editor::OpenComponentEditorAct, component_list::OpenComponentListAct,
    },
    dirs_paths::data_path,
    file::{load_toml, save_toml_with_header},
    history::{history_viewer::OpenHistoryViewerAct, HistoryAct},
    info_windows::InfoWindowsAct,
    keymaps::{
        key_list::KEY_LIST,
        settings_editor::{OpenKeymapSettingsAct, KEYMAP_MENU},
    },
    misc_config::settings_editor::OpenMiscSettingsAct,
    notification::viewer::OpenNotifLogViewerAct,
    project::{events::ProjectAct, project_editor::OpenProjectEditorAct},
    state::{ChangeStateAct, EditorState},
    ui::{panel::menu::OpenAllSettingsAct, tilemap::settings_editor::TileSettingsAct},
    window::settings_editor::OpenWindowSettingsAct,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeymapAction {
    ChangeState(EditorState),
    Undo,
    Redo,
    Quit,
    SelectProjectFolder,
    SaveProject,
    ReloadProject,
    TileSettings,
    WindowSettings,
    KeymapSettings,
    MiscSettings,
    AllSettings,
    ComponentEditor,
    Project,
    ComponentList,
    History,
    NotifLog,
}

impl KeymapAction {
    #[must_use]
    pub fn action(self) -> Action {
        match self {
            Self::ChangeState(state) => Action::new(ChangeStateAct(state)),
            Self::Undo => Action::new(HistoryAct::Undo),
            Self::Redo => Action::new(HistoryAct::Redo),
            Self::Quit => Action::new(InfoWindowsAct::Quit(false)),
            Self::SelectProjectFolder => Action::new(ProjectAct::SelectFolder),
            Self::SaveProject => Action::new(ProjectAct::Save(false)),
            Self::ReloadProject => Action::new(ProjectAct::GetNamespaces),
            Self::TileSettings => Action::new(TileSettingsAct::Open),
            Self::WindowSettings => Action::new(OpenWindowSettingsAct),
            Self::KeymapSettings => Action::new(OpenKeymapSettingsAct),
            Self::MiscSettings => Action::new(OpenMiscSettingsAct),
            Self::AllSettings => Action::new(OpenAllSettingsAct),
            Self::ComponentEditor => Action::new(OpenComponentEditorAct),
            Self::Project => Action::new(OpenProjectEditorAct),
            Self::ComponentList => Action::new(OpenComponentListAct),
            Self::History => Action::new(OpenHistoryViewerAct),
            Self::NotifLog => Action::new(OpenNotifLogViewerAct),
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
                (KeymapAction::Quit, KeyCode::Escape),
                (KeymapAction::SelectProjectFolder, KeyCode::KeyO),
                (KeymapAction::SaveProject, KeyCode::KeyS),
                (KeymapAction::ReloadProject, KeyCode::KeyR),
                (KeymapAction::TileSettings, KeyCode::KeyT),
                (KeymapAction::WindowSettings, KeyCode::KeyW),
                (KeymapAction::KeymapSettings, KeyCode::KeyK),
                (KeymapAction::MiscSettings, KeyCode::KeyM),
                (KeymapAction::AllSettings, KeyCode::KeyA),
                (KeymapAction::ComponentEditor, KeyCode::KeyC),
                (KeymapAction::Project, KeyCode::KeyP),
                (KeymapAction::ComponentList, KeyCode::KeyL),
                (KeymapAction::History, KeyCode::KeyH),
                (KeymapAction::NotifLog, KeyCode::KeyN),
            ]
            .into_iter()
            .collect(),
        )
    }
}

impl KeymapSettings {
    pub fn load() -> Self {
        if !data_path("keymap_settings.toml").exists() {
            let s = Self::default();
            let _ = s.save();
            return s;
        }
        match load_toml(&data_path("keymap_settings.toml"), Some("keymap settings"))
            .and_then(|a| Self::from_serializable(&a))
        {
            Ok(str) => {
                info!("Found keymap settings file");
                str
            }
            Err(e) => {
                info!("Couldn't open or parse keymap settings file: {e:?}");

                Self::default()
            }
        }
    }
    pub fn save(&self) -> eyre::Result<()> {
        save_toml_with_header(&self.as_serializable()?, &data_path("keymap_settings.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#keymap_settingstoml", Some("keymap settings"))
    }

    pub fn as_serializable(&self) -> eyre::Result<HashMap<&str, HashMap<String, String>>> {
        let default = Self::default();
        KEYMAP_MENU
            .iter()
            .map(|(cat, menu)| {
                menu.iter()
                    .map(|a| {
                        Ok((
                            &a.0,
                            default.0.get(&a.0).ok_or_eyre(format!(
                                "Action {:?} not registered in default keymap",
                                a.0
                            ))?,
                            self.0.get(&a.0).ok_or_eyre(format!(
                                "Action {:?} not registered in custom keymap",
                                a.0
                            ))?,
                        ))
                    })
                    .filter_ok(|(_, default_key, custom_key)| default_key != custom_key)
                    .map_ok(|(action, _, custom_key)| {
                        (format!("{action:?}"), format!("{custom_key:?}"))
                    })
                    .collect::<Result<_, _>>()
                    .map(|a| (*cat, a))
            })
            .collect()
    }

    pub fn from_serializable(o: &HashMap<String, HashMap<String, String>>) -> eyre::Result<Self> {
        let mut s = Self::default();
        for menu in o.values() {
            for (action, key) in menu {
                let action =
                    s.0.keys()
                        .find(|a| format!("{a:?}") == *action)
                        .ok_or_eyre(format!("Invalid action {action} in custom keymap"))?;
                let key = KEY_LIST
                    .iter()
                    .find(|a| format!("{a:?}") == *key)
                    .ok_or_eyre(format!("Invalid key {key} in custom keymap"))?;
                s.0.insert(*action, *key);
            }
        }
        Ok(s)
    }
}

pub static INIT_KEYMAP_SETTINGS: Lazy<KeymapSettings> = Lazy::new(KeymapSettings::load);
