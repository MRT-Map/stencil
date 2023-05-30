use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, window::PrimaryWindow,
    winit::WinitWindows,
};
use bevy_mod_picking::prelude::*;
use bevy_mouse_tracking::{prelude::*, MainCamera};
use winit::window::Icon;

use crate::state::LoadingState;

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_camera(mut commands: Commands) {
    info!("Spawning camera");
    commands
        .spawn(Camera2dBundle::new_with_far(1e5))
        .insert(MainCamera)
        .insert(UiCameraConfig { show_ui: true })
        .insert(RaycastPickCamera::default())
        .add(InitWorldTracking);

    commands.insert_resource(NextState(Some(LoadingState::SpawnCamera.next())));
}
