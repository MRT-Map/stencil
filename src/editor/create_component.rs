use bevy::prelude::*;
use bevy_mouse_tracking_plugin::MousePos;

pub fn mouse_button_input(
    buttons: Res<Input<MouseButton>>,
    mouse_pos: Res<MousePos>,
) {
    if buttons.just_released(MouseButton::Left) {
        println!("{:?}", mouse_pos);
    }
}