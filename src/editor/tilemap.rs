use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    editor::ui::HoveringOverGui,
    types::{zoom::Zoom, EditorState, Label},
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
                    .run_if_not(|hovering_over_gui: Res<HoveringOverGui>| hovering_over_gui.0)
                    .with_system(mouse_nav::mouse_drag)
                    .with_system(mouse_nav::mouse_zoom)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .with_system(tile::show_tiles)
                    .into(),
            );
    }
}
