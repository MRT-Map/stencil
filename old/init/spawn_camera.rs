use bevy::prelude::*;

use crate::state::LoadingState;

pub fn spawn_camera_sy(mut commands: Commands) {
    info!("Spawning camera");
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            far: 1e5,
            ..OrthographicProjection::default_2d()
        }),
        MeshPickingCamera,
    ));
    //.insert(UiCameraConfig { show_ui: true })
    //.insert(RaycastPickCamera::default())

    commands.insert_resource(NextState::Pending(LoadingState::SpawnCamera.next()));
}
