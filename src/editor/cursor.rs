use crate::{EditorState, HoveringOverGui};
use bevy::prelude::*;
use bevy_mouse_tracking_plugin::MousePos;
use iyes_loopless::prelude::*;

pub fn cursor_icon(
    buttons: Res<Input<MouseButton>>,
    mouse_pos: Res<MousePos>,
    mut windows: ResMut<Windows>,
    state: Res<CurrentState<EditorState>>,
    hovering: Res<HoveringOverGui>,
) {
    if !hovering.0 {
        windows.primary_mut().set_cursor_icon(match state.0 {
            EditorState::Loading => CursorIcon::Wait,
            EditorState::Idle => {
                if buttons.pressed(MouseButton::Left) {
                    CursorIcon::Grabbing
                } else {
                    CursorIcon::Grab
                }
            }
            EditorState::CreatingComponent(_) => CursorIcon::Crosshair,
            EditorState::EditingNodes => CursorIcon::Hand,
            EditorState::MovingComponent => CursorIcon::Hand,
            EditorState::RotatingComponent => CursorIcon::Hand,
            EditorState::DeletingComponent => CursorIcon::Hand,
        });
    }
    if buttons.just_released(MouseButton::Left) {
        println!("{:?}", mouse_pos);
    }
}
