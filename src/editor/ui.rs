mod component_panel;
mod menu;
mod toolbar;

use bevy::prelude::*;
use iyes_loopless::condition::ConditionSet;

use crate::types::{EditorState};

#[derive(Default)]
pub struct HoveringOverGui(pub bool);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveringOverGui>()
            .add_stage_before(CoreStage::Update, "ui", SystemStage::single_threaded())
            .add_system_set_to_stage(
                "ui",
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label("ui_menu")
                    .before("ui_component_panel")
                    .with_system(menu::ui)
                    .into(),
            )
            .add_system_set_to_stage(
                "ui",
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label("ui_component_panel")
                    .after("ui_menu")
                    .before("ui_toolbar")
                    .with_system(component_panel::ui)
                    .into(),
            )
            .add_system_set_to_stage(
                "ui",
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label("ui_toolbar")
                    .after("ui_component_panel")
                    .with_system(toolbar::ui)
                    .into(),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                |mut hovering_over_gui: ResMut<HoveringOverGui>| hovering_over_gui.0 = false,
            );
    }
}
