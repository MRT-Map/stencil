use bevy::prelude::*;
use iyes_loopless::condition::ConditionSet;

use crate::{
    misc::{CustomStage, EditorState},
    ui::component_panel::PrevNamespaceUsed,
};

pub mod component_panel;
pub mod file_explorer;
pub mod menu;
pub mod popup;
pub mod toolbar;

#[derive(Default, Resource)]
pub struct HoveringOverGui(pub bool);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveringOverGui>()
            .init_resource::<PrevNamespaceUsed>()
            .add_stage_before(
                CoreStage::Update,
                CustomStage::Ui,
                SystemStage::single_threaded(),
            )
            .add_system_set_to_stage(
                CustomStage::Ui,
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label("ui_menu")
                    .before("ui_component_panel")
                    .with_system(menu::ui_sy)
                    .into(),
            )
            .add_system_set_to_stage(
                CustomStage::Ui,
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label("ui_component_panel")
                    .after("ui_menu")
                    .before("ui_toolbar")
                    .with_system(component_panel::ui_sy)
                    .into(),
            )
            .add_system_set_to_stage(
                CustomStage::Ui,
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label("ui_toolbar")
                    .after("ui_component_panel")
                    .with_system(toolbar::ui_sy)
                    .into(),
            )
            .add_system_to_stage(
                CoreStage::Last,
                |mut hovering_over_gui: ResMut<HoveringOverGui>| hovering_over_gui.0 = false,
            )
            .add_plugin(popup::PopupPlugin);
    }
}
