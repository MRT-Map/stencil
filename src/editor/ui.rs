mod component_panel;
mod menu;
mod toolbar;

use bevy::prelude::*;
use iyes_loopless::condition::ConditionSet;

use crate::types::{EditorState, Label};

#[derive(Default)]
pub struct HoveringOverGui(pub bool);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveringOverGui>()
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label(Label::MenuUi)
                    .with_system(menu::ui)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label(Label::ComponentPanelUi)
                    .after(Label::MenuUi)
                    .before(Label::ToolbarUi)
                    .with_system(component_panel::ui)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label(Label::ToolbarUi)
                    .after(Label::ComponentPanelUi)
                    .before(Label::Controls)
                    .with_system(toolbar::ui)
                    .into(),
            )
            .add_system(
                (|mut hovering: ResMut<HoveringOverGui>| hovering.0 = false).label(Label::Cleanup),
            );
    }
}
