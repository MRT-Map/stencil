use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bimap::BiHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    component_actions::undo_redo::UndoRedoAct,
    info_windows::InfoWindowsAct,
    load_save::LoadSaveAct,
    misc::Action,
    state::{ChangeStateAct, EditorState, IntoSystemConfigExt},
    ui::tilemap::settings_editor::OpenTileSettingsAct,
    window_settings::settings_editor::OpenWindowSettingsAct,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HotkeyActions {
    ChangeState(EditorState),
    Undo,
    Redo,
    LoadNamespaces,
    SaveNamespaces,
    TileSettings,
    WindowSettings,
    Quit,
}
impl HotkeyActions {
    #[must_use]
    pub fn action(self) -> Action {
        match self {
            Self::ChangeState(state) => Action::new(ChangeStateAct(state)),
            Self::Undo => Action::new(UndoRedoAct::Undo),
            Self::Redo => Action::new(UndoRedoAct::Redo),
            Self::LoadNamespaces => Action::new(LoadSaveAct::Load),
            Self::SaveNamespaces => Action::new(LoadSaveAct::Save),
            Self::TileSettings => Action::new(OpenTileSettingsAct),
            Self::WindowSettings => Action::new(OpenWindowSettingsAct),
            Self::Quit => Action::new(InfoWindowsAct::Quit(false)),
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HotkeySettings(pub BiHashMap<HotkeyActions, KeyCode>);

impl Default for HotkeySettings {
    fn default() -> Self {
        Self(
            [
                (
                    HotkeyActions::ChangeState(EditorState::Idle),
                    KeyCode::Digit1,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::EditingNodes),
                    KeyCode::Digit2,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::DeletingComponent),
                    KeyCode::Digit3,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::CreatingPoint),
                    KeyCode::Digit4,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::CreatingLine),
                    KeyCode::Digit5,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::CreatingArea),
                    KeyCode::Digit6,
                ),
                (HotkeyActions::Undo, KeyCode::KeyU),
                (HotkeyActions::Redo, KeyCode::KeyY),
                (HotkeyActions::LoadNamespaces, KeyCode::KeyL),
                (HotkeyActions::SaveNamespaces, KeyCode::KeyS),
                (HotkeyActions::TileSettings, KeyCode::KeyT),
                (HotkeyActions::WindowSettings, KeyCode::KeyW),
                (HotkeyActions::Quit, KeyCode::Escape),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn hotkey_sy(
    mut actions: EventWriter<Action>,
    hotkey_settings: Res<HotkeySettings>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ctx: EguiContexts,
) {
    for (action, key) in &hotkey_settings.0 {
        if keys.just_released(*key) && ctx.ctx_mut().memory(|a| a.focused().is_none()) {
            info!(?action, ?key, "Processing hotkey");
            actions.send(action.action());
        }
    }
}

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HotkeySettings::default())
            .add_systems(Update, hotkey_sy.run_if_not_loading());
    }
}
