use bevy::prelude::*;
use zoom::Zoom;

use crate::{
    state::IntoSystemConfigExt,
    ui::{
        EguiContextPass, UiSet,
        map::{settings::INIT_TILE_SETTINGS, tiles::PendingTiles},
    },
};

pub mod mouse_nav;
pub mod settings;
pub mod settings_editor;
pub mod tiles;
pub mod utils;
pub mod window;
pub mod zoom;

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
                (mouse_nav::mouse_pan_sy, mouse_nav::mouse_zoom_sy).in_set(RenderingSet::Mouse),
            )
            .add_systems(
                EguiContextPass,
                (tiles::show_tiles_sy, settings_editor::tile_settings_dialog)
                    .in_set(RenderingSet::Tiles),
            )
            .add_observer(settings_editor::on_tile_settings);
    }
}
