use bevy::{
    app::{App, Plugin},
    prelude::*,
};

use crate::{
    state::EditorState,
    tile::{settings::INIT_TILE_SETTINGS, zoom::Zoom},
    ui::{HoveringOverGui, UiSet},
};

pub mod mouse_nav;
pub mod settings;
pub mod tile;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum RenderingSet {
    Mouse,
    Tiles,
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Zoom(INIT_TILE_SETTINGS.init_zoom))
            .insert_resource(INIT_TILE_SETTINGS.to_owned())
            .configure_set(
                RenderingSet::Mouse
                    .run_if(not(resource_exists_and_equals(HoveringOverGui(true))))
                    .in_set(UiSet::Tiles),
            )
            .configure_set(RenderingSet::Tiles.run_if(not(in_state(EditorState::Loading))))
            .add_systems(
                (mouse_nav::mouse_drag_sy, mouse_nav::mouse_zoom_sy).in_set(RenderingSet::Mouse),
            )
            .add_systems(
                (tile::show_tiles_sy, settings::tile_settings_msy).in_set(RenderingSet::Tiles),
            );
    }
}
