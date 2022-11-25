use bevy::prelude::*;
use bevy_mod_picking::PickingCameraBundle;
use bevy_mouse_tracking_plugin::{prelude::*, MainCamera};
use iyes_loopless::prelude::*;

use crate::{
    error_handling::ack_panic_sy,
    misc::{state_changer_asy, Action, EditorState},
    pla2::skin::{get_skin_sy, Skin},
};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(EditorState::Loading)
            .init_resource::<Skin>()
            .add_event::<Action>()
            .add_system(get_skin_sy)
            .add_system(state_changer_asy)
            .add_system(ack_panic_sy)
            .add_exit_system(EditorState::Loading, setup_sy);
    }
}

fn setup_sy(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::new_with_far(1e5))
        .insert(MainCamera)
        .insert(UiCameraConfig { show_ui: true })
        .insert(PickingCameraBundle::default())
        .add_world_tracking();
}
