use bevy::{prelude::*, window::PrimaryWindow, winit::WinitWindows};
use bevy_mod_picking::PickingCameraBundle;
use bevy_mouse_tracking_plugin::{prelude::*, MainCamera};
use winit::window::Icon;

use crate::{
    error_handling::ack_panic_sy,
    misc::{state_changer_asy, Action, EditorState},
    pla2::skin::{get_skin_sy, Skin},
};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<EditorState>()
            .init_resource::<Skin>()
            .add_event::<Action>()
            .add_system(get_skin_sy)
            .add_system(state_changer_asy)
            .add_startup_system(ack_panic_sy)
            .add_system(setup_sy.in_schedule(OnExit(EditorState::Loading)));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn setup_sy(
    mut commands: Commands,
    windows: NonSend<WinitWindows>,
    primary_id: Query<Entity, With<PrimaryWindow>>,
) {
    commands
        .spawn(Camera2dBundle::new_with_far(1e5))
        .insert(MainCamera)
        .insert(UiCameraConfig { show_ui: true })
        .insert(PickingCameraBundle::default())
        .add(InitWorldTracking);

    // https://bevy-cheatbook.github.io/window/icon.html
    let primary = windows.get_window(primary_id.single()).unwrap();

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(include_bytes!(
            "../build/macos/AppIcon.iconset/icon_512x512@2x.png"
        ))
        .unwrap()
        .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    primary.set_window_icon(Some(icon));
}
