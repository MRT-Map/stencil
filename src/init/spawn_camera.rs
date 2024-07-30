use bevy::prelude::*;
use bevy_mouse_tracking::{prelude::*, MainCamera};

use crate::state::LoadingState;

pub fn spawn_camera_sy(mut commands: Commands) {
    info!("Spawning camera");
    commands
        .spawn(Camera2dBundle::new_with_far(1e5))
        .insert(MainCamera)
        //.insert(UiCameraConfig { show_ui: true })
        //.insert(RaycastPickCamera::default())
        .add(InitWorldTracking);

    commands.insert_resource(NextState::Pending(LoadingState::SpawnCamera.next()));
}
