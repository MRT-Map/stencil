pub mod compat;
pub mod load_assets;
mod load_skin;
pub mod set_icon;
pub mod spawn_camera;
pub mod unzip_assets;

use bevy::prelude::*;
use load_skin::get_skin_sy;

use crate::{
    error_handling::ack_panic_sy,
    misc::{cache_path, Action},
    pla2::skin::Skin,
    state::{state_changer_asy, EditorState, LoadingState},
    ui::tilemap::settings::INIT_TILE_SETTINGS,
};

pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EditorState>()
            .init_state::<LoadingState>()
            .init_resource::<Skin>()
            .add_event::<Action>()
            .add_systems(Update, state_changer_asy)
            .add_systems(Startup, ack_panic_sy);
        app.add_systems(OnEnter(LoadingState::SetIcon), set_icon::set_icon_sy)
            .add_systems(
                OnEnter(LoadingState::UnzipAssets),
                unzip_assets::unzip_assets_sy,
            )
            .add_plugins(load_assets::LoadAssetsPlugin)
            .add_systems(OnEnter(LoadingState::Compat), compat::compat_sy)
            .add_systems(Update, get_skin_sy.run_if(in_state(LoadingState::LoadSkin)))
            .add_systems(
                OnEnter(LoadingState::SpawnCamera),
                spawn_camera::spawn_camera,
            )
            .add_systems(OnEnter(LoadingState::Done), done_sy);
    }
}

fn done_sy(mut commands: Commands) {
    if INIT_TILE_SETTINGS.clear_cache_on_startup {
        info!("Removing previous tile cache");
        let _ = std::fs::remove_dir_all(cache_path("tile-cache"));
    }

    info!("Transitioning out of idle");
    commands.insert_resource(NextState(Some(EditorState::Idle)));
}
