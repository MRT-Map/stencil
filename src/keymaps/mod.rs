mod key_list;
pub mod settings;
pub mod settings_editor;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use settings::KeymapSettings;

use crate::{action::Action, keymaps::settings::INIT_KEYMAP_SETTINGS, state::IntoSystemConfigExt};

#[allow(clippy::needless_pass_by_value)]
pub fn keymap_sy(
    mut actions: EventWriter<Action>,
    hotkey_settings: Res<KeymapSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ctx: EguiContexts,
) {
    for (action, key) in &hotkey_settings.0 {
        if keys.just_released(*key)
            && ctx
                .try_ctx_mut()
                .map_or(true, |a| a.memory(|a| a.focused().is_none()))
        {
            info!(?action, ?key, "Processing hotkey");
            actions.send(action.action());
        }
    }
}

pub struct KeymapPlugin;

impl Plugin for KeymapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(INIT_KEYMAP_SETTINGS.to_owned())
            .add_systems(Update, keymap_sy.run_if_not_loading())
            .add_systems(Update, settings_editor::keymap_settings_asy);
    }
}
