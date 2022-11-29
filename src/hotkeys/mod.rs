use bevy::prelude::*;
use bevy_egui::EguiContext;
use bimap::BiHashMap;
use iyes_loopless::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    component_actions::undo_redo::UndoRedoAct,
    info_windows::InfoWindowsAct,
    load_save::LoadSaveAct,
    misc::{Action, ChangeStateAct, EditorState},
    pla2::component::ComponentType,
    tilemap::settings::TileSettingsAct,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HotkeyActions {
    ChangeState(EditorState),
    Undo,
    Redo,
    LoadNamespaces,
    SaveNamespaces,
    TileSettings,
    Quit,
}
impl HotkeyActions {
    pub fn action(self) -> Action {
        match self {
            Self::ChangeState(state) => Box::new(ChangeStateAct(state)),
            Self::Undo => Box::new(UndoRedoAct::Undo),
            Self::Redo => Box::new(UndoRedoAct::Redo),
            Self::LoadNamespaces => Box::new(LoadSaveAct::Load),
            Self::SaveNamespaces => Box::new(LoadSaveAct::Save),
            Self::TileSettings => Box::new(TileSettingsAct::Open),
            Self::Quit => Box::new(InfoWindowsAct::Quit(false)),
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HotkeySettings(pub BiHashMap<HotkeyActions, KeyCode>);

impl Default for HotkeySettings {
    fn default() -> Self {
        Self(
            [
                (HotkeyActions::ChangeState(EditorState::Idle), KeyCode::Key1),
                (
                    HotkeyActions::ChangeState(EditorState::EditingNodes),
                    KeyCode::Key2,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::DeletingComponent),
                    KeyCode::Key3,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::CreatingComponent(
                        ComponentType::Point,
                    )),
                    KeyCode::Key4,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::CreatingComponent(ComponentType::Line)),
                    KeyCode::Key5,
                ),
                (
                    HotkeyActions::ChangeState(EditorState::CreatingComponent(ComponentType::Area)),
                    KeyCode::Key6,
                ),
                (HotkeyActions::Undo, KeyCode::U),
                (HotkeyActions::Redo, KeyCode::Y),
                (HotkeyActions::LoadNamespaces, KeyCode::L),
                (HotkeyActions::SaveNamespaces, KeyCode::S),
                (HotkeyActions::TileSettings, KeyCode::T),
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
    keys: Res<Input<KeyCode>>,
    mut ctx: ResMut<EguiContext>,
) {
    for (action, key) in &hotkey_settings.0 {
        if keys.just_released(*key) && ctx.ctx_mut().memory().focus().is_none() {
            info!(?action, ?key, "Processing hotkey");
            actions.send(action.action());
        }
    }
}

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HotkeySettings::default())
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .with_system(hotkey_sy)
                    .into(),
            );
    }
}
