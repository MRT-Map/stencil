mod key_list;
pub mod settings;
pub mod settings_editor;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use settings::KeymapSettings;

use crate::{
    keymaps::settings_editor::keymap_settings_msy, misc::Action, state::IntoSystemConfigExt,
};

#[allow(clippy::needless_pass_by_value)]
pub fn keymap_sy(
    mut actions: EventWriter<Action>,
    hotkey_settings: Res<KeymapSettings>,
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

pub struct KeymapPlugin;

impl Plugin for KeymapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeymapSettings::default())
            .add_systems(Update, keymap_sy.run_if_not_loading())
            .add_systems(Update, keymap_settings_msy);
    }
}
