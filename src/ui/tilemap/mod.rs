use bevy::prelude::*;

use crate::{
    state::IntoSystemSetConfigExt,
    tile::zoom::Zoom,
    ui::{tilemap::settings::INIT_TILE_SETTINGS, UiSchedule, UiSet},
};

pub mod mouse_nav;
pub mod settings;
pub mod settings_window;
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
            .configure_sets(
                UiSchedule,
                RenderingSet::Mouse
                    .run_if_not_loading()
                    .in_set(UiSet::Tiles),
            )
            .configure_sets(UiSchedule, RenderingSet::Tiles.run_if_not_loading())
            .add_systems(
                UiSchedule,
                (mouse_nav::mouse_drag_sy, mouse_nav::mouse_zoom_sy).in_set(RenderingSet::Mouse),
            )
            .add_systems(
                UiSchedule,
                (tile::show_tiles_sy, settings_window::tile_settings_msy)
                    .in_set(RenderingSet::Tiles),
            );
    }
}
