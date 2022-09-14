use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{EditorState, Label};

pub mod component_panel;
pub mod creating_component;
pub mod cursor;
pub mod menu;
pub mod selecting_component;
pub mod shadow;
pub mod toolbar;

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
