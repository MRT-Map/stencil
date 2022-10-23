use bevy::prelude::*;
use bevy_mod_picking::PickingCameraBundle;
use bevy_mouse_tracking_plugin::{prelude::*, MainCamera};
use iyes_loopless::prelude::*;

use crate::pla2::{
    component::ComponentType,
    skin::{request_skin_sy, retrieve_skin_sy, Skin},
};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(EditorState::Loading)
            .init_resource::<Skin>()
            .add_startup_system(request_skin_sy)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(EditorState::Loading)
                    .with_system(retrieve_skin_sy)
                    .into(),
            )
            .add_exit_system(EditorState::Loading, setup_sy);
    }
}

fn setup_sy(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera)
        .insert(UiCameraConfig { show_ui: true })
        .insert_bundle(PickingCameraBundle::default())
        .add_world_tracking();
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditorState {
    #[default]
    Loading,
    Idle,
    CreatingComponent(ComponentType),
    EditingNodes,
    DeletingComponent,
}
