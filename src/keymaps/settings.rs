use std::collections::HashMap;

use bevy::prelude::{Commands, KeyCode, Resource};
use eyre::OptionExt;
use itertools::Itertools;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    component::panels::{
        component_editor::OpenComponentEditorEv, component_list::OpenComponentListEv,
    },
    dirs_paths::data_path,
    file::{load_toml, save_toml_with_header},
    history::{history_viewer::OpenHistoryViewerEv, HistoryEv},
    info_windows::InfoWindowsEv,
    keymaps::{
        key_list::KEY_LIST,
        settings_editor::{OpenKeymapSettingsEv, KEYMAP_MENU},
    },
    misc_config::settings_editor::OpenMiscSettingsEv,
    project::{events::ProjectEv, project_editor::OpenProjectEditorEv},
    state::{ChangeStateEv, EditorState},
    ui::{
        notif::viewer::OpenNotifLogViewerEv, panel::menu::OpenAllSettingsEv,
        tilemap::settings_editor::TileSettingsEv,
    },
    window::settings_editor::OpenWindowSettingsEv,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeymapAction {
    ChangeState(EditorState),
    Undo,
    Redo,
    Quit,
    OpenProject,
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
    pub fn trigger_action(self, commands: &mut Commands) {
        match self {
            Self::ChangeState(state) => commands.trigger(ChangeStateEv(state)),
            Self::Undo => commands.trigger(HistoryEv::Undo),
            Self::Redo => commands.trigger(HistoryEv::Redo),
            Self::Quit => commands.trigger(InfoWindowsEv::Quit(false)),
            Self::OpenProject => commands.trigger(ProjectEv::Open),
            Self::SaveProject => commands.trigger(ProjectEv::Save(false)),
            Self::ReloadProject => commands.trigger(ProjectEv::Reload),
            Self::TileSettings => commands.trigger(TileSettingsEv::Open),
            Self::WindowSettings => commands.trigger(OpenWindowSettingsEv),
            Self::KeymapSettings => commands.trigger(OpenKeymapSettingsEv),
            Self::MiscSettings => commands.trigger(OpenMiscSettingsEv),
            Self::AllSettings => commands.trigger(OpenAllSettingsEv),
            Self::ComponentEditor => commands.trigger(OpenComponentEditorEv),
            Self::Project => commands.trigger(OpenProjectEditorEv),
            Self::ComponentList => commands.trigger(OpenComponentListEv),
            Self::History => commands.trigger(OpenHistoryViewerEv),
            Self::NotifLog => commands.trigger(OpenNotifLogViewerEv),
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
                (KeymapAction::OpenProject, KeyCode::KeyO),
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
