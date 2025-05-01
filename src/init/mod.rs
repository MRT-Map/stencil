pub mod compat;
pub mod load_assets;
mod load_fonts;
mod load_skin;
pub mod set_icon;
pub mod spawn_camera;
pub mod unzip_assets;
mod welcome;

use bevy::prelude::*;
use load_skin::get_skin_sy;

use crate::{
    component::skin::Skin,
    dirs_paths::cache_path,
    file::safe_delete,
    init::load_fonts::get_fonts_sy,
    panic::ack_panic_sy,
    state::{on_state_change, EditorState, LoadingState},
    ui::{map::settings::INIT_TILE_SETTINGS, panel::status::Status},
};

pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EditorState>()
            .init_state::<LoadingState>()
            .init_resource::<Skin>()
            .add_observer(on_state_change)
            .add_systems(OnExit(EditorState::Loading), ack_panic_sy);
        app.add_systems(OnEnter(LoadingState::SetIcon), set_icon::set_icon_sy)
            .add_systems(
                OnEnter(LoadingState::UnzipAssets),
                unzip_assets::unzip_assets_sy,
            )
            .add_plugins(load_assets::LoadAssetsPlugin)
            .add_systems(OnEnter(LoadingState::Compat), compat::compat_sy)
            .add_systems(Update, get_skin_sy.run_if(in_state(LoadingState::LoadSkin)))
            .add_systems(
                Update,
                get_fonts_sy.run_if(in_state(LoadingState::LoadFonts)),
            )
            .add_systems(
                OnEnter(LoadingState::SpawnCamera),
                spawn_camera::spawn_camera_sy,
            )
            .add_systems(OnEnter(LoadingState::Welcome), welcome::welcome_sy)
            .add_systems(OnEnter(LoadingState::Done), done_sy);
    }
}

fn done_sy(mut commands: Commands, mut status: ResMut<Status>) {
    if INIT_TILE_SETTINGS.clear_cache_on_startup {
        info!("Removing previous tile cache");
        let _ = safe_delete(&cache_path("tile-cache"), None);
    }

    status.0 = format!("Welcome to Stencil v{}", env!("CARGO_PKG_VERSION")).into();

    info!("Transitioning out of idle");
    commands.insert_resource(NextState::Pending(EditorState::Idle));
}
