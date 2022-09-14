use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    editor::HoveringOverGui, mouse_drag, mouse_zoom, show_tiles, EditorState, Label, Zoom,
};

pub mod mouse_nav;
pub mod tile;
pub mod utils;

pub struct RenderingPlugin;
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Zoom(7.0))
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .label(Label::Controls)
                    .after(Label::ToolbarUi)
                    .before(Label::Cleanup)
                    .run_if_not(|hovering: Res<HoveringOverGui>| hovering.0)
                    .with_system(mouse_drag)
                    .with_system(mouse_zoom)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .with_system(show_tiles)
                    .into(),
            );
    }
}
