use bevy::prelude::*;

use crate::{
    state::IntoSystemConfigExt,
    tile::zoom::Zoom,
    ui::{
        tilemap::{settings::INIT_TILE_SETTINGS, tile::PendingTiles},
        EguiContextPass, UiSet,
    },
};

pub mod mouse_nav;
pub mod settings;
pub mod settings_editor;
pub mod tile;
pub mod window;

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
            .init_resource::<PendingTiles>()
            .configure_sets(
                EguiContextPass,
                RenderingSet::Mouse
                    .run_if_not_loading()
                    .in_set(UiSet::Tiles),
            )
            .configure_sets(EguiContextPass, RenderingSet::Tiles.run_if_not_loading())
            .add_systems(
                Update,
                (mouse_nav::mouse_drag_sy, mouse_nav::mouse_zoom_sy).in_set(RenderingSet::Mouse),
            )
            .add_systems(
                EguiContextPass,
                (tile::show_tiles_sy, settings_editor::tile_settings_dialog)
                    .in_set(RenderingSet::Tiles),
            )
            .add_observer(settings_editor::on_tile_settings);
    }
}
