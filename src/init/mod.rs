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
};

pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<EditorState>()
            .add_state::<LoadingState>()
            .init_resource::<Skin>()
            .add_event::<Action>()
            .add_system(state_changer_asy)
            .add_startup_system(ack_panic_sy);
        app.add_system(set_icon::set_icon_sy.in_schedule(OnEnter(LoadingState::SetIcon)))
            .add_system(
                unzip_assets::unzip_assets_sy.in_schedule(OnEnter(LoadingState::UnzipAssets)),
            )
            .add_plugin(load_assets::LoadAssetsPlugin)
            .add_system(compat::compat_sy.in_schedule(OnEnter(LoadingState::Compat)))
            .add_system(get_skin_sy.run_if(in_state(LoadingState::LoadSkin)))
            .add_system(spawn_camera::spawn_camera.in_schedule(OnEnter(LoadingState::SpawnCamera)))
            .add_system(done_sy.in_schedule(OnEnter(LoadingState::Done)));
    }
}

fn done_sy(mut commands: Commands) {
    //info!("Removing previous tile cache");
    //let _ = std::fs::remove_dir_all(cache_path("tile-cache"));

    info!("Transitioning out of idle");
    commands.insert_resource(NextState(Some(EditorState::Idle)));
}
