use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{EditorState, HoveringOverGui};

pub fn cursor_icon(
    buttons: Res<Input<MouseButton>>,
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
}
