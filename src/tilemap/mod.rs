use bevy::prelude::*;
use iyes_loopless::prelude::*;
use zoom::Zoom;

use crate::{misc::EditorState, ui::HoveringOverGui};

pub mod bundle;
pub mod mouse_nav;
pub mod settings;
pub mod tile;
pub mod tile_coord;
pub mod utils;
pub mod zoom;

pub struct RenderingPlugin;
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Zoom(7.0))
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .run_if_not(|hovering_over_gui: Res<HoveringOverGui>| hovering_over_gui.0)
                    .with_system(mouse_nav::mouse_drag_sy)
                    .with_system(mouse_nav::mouse_zoom_sy)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(EditorState::Loading)
                    .with_system(tile::show_tiles_sy)
                    .with_system(settings::tile_settings_msy)
                    .into(),
            );
    }
}
