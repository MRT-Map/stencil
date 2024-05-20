pub mod compat;
pub mod load_assets;
mod load_skin;
pub mod set_icon;
pub mod spawn_camera;
pub mod unzip_assets;
mod welcome;

use bevy::prelude::*;
use egui_notify::Toasts;
use load_skin::get_skin_sy;

use crate::{
    component::skin::Skin,
    error::{
        log::{update_error_log_sy, NotifToasts},
        panic::ack_panic_sy,
    },
    misc::{cache_path, Action},
    state::{state_changer_asy, EditorState, IntoSystemConfigExt, LoadingState},
    ui::{panel::status::Status, tilemap::settings::INIT_TILE_SETTINGS},
};

pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EditorState>()
            .init_state::<LoadingState>()
            .init_resource::<Skin>()
            .add_event::<Action>()
            .init_resource::<NotifToasts>()
            .add_systems(Update, state_changer_asy)
            .add_systems(Startup, ack_panic_sy)
            .add_systems(Update, update_error_log_sy);
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
                spawn_camera::spawn_camera_sy,
            )
            .add_systems(OnEnter(LoadingState::Welcome), welcome::welcome_sy)
            .add_systems(OnEnter(LoadingState::Done), done_sy);
    }
}

fn done_sy(mut commands: Commands, mut status: ResMut<Status>) {
    if INIT_TILE_SETTINGS.clear_cache_on_startup {
        info!("Removing previous tile cache");
        let _ = std::fs::remove_dir_all(cache_path("tile-cache"));
    }

    status.0 = format!("Welcome to Stencil v{}", env!("CARGO_PKG_VERSION")).into();

    info!("Transitioning out of idle");
    commands.insert_resource(NextState(Some(EditorState::Idle)));
}
