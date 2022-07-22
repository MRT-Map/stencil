use bevy::prelude::*;
use bevy_mouse_tracking_plugin::MousePos;
use iyes_loopless::prelude::*;
use crate::EditorState;

pub fn mouse_button_input(
    buttons: Res<Input<MouseButton>>,
    mouse_pos: Res<MousePos>,
    mut windows: ResMut<Windows>,
    state: Res<CurrentState<EditorState>>
) {
    if let EditorState::CreatingComponent(_) = *state { } else { return }
    windows.primary_mut().set_cursor_icon(CursorIcon::Crosshair);
    if buttons.just_released(MouseButton::Left) {
        println!("{:?}", mouse_pos);
    }
}
