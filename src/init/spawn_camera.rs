use bevy::prelude::*;
use bevy_mouse_tracking::{prelude::*, MainCamera};

use crate::state::LoadingState;

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_camera(mut commands: Commands) {
    info!("Spawning camera");
    commands
        .spawn(Camera2dBundle::new_with_far(1e5))
        .insert(MainCamera)
        //.insert(UiCameraConfig { show_ui: true })
        //.insert(RaycastPickCamera::default())
        .add(InitWorldTracking);

    commands.insert_resource(NextState(Some(LoadingState::SpawnCamera.next())));
}
