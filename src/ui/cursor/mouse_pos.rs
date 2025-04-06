use std::ops::Deref;

use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Debug, Resource, Clone, Copy, PartialEq, Default)]
pub struct MousePos(Vec2);

impl Deref for MousePos {
    type Target = Vec2;

    fn deref(&self) -> &Vec2 {
        &self.0
    }
}

#[derive(Debug, Resource, Clone, Copy, PartialEq, Default)]
pub struct MousePosWorld(pub(crate) Vec2);

impl Deref for MousePosWorld {
    type Target = Vec2;

    fn deref(&self) -> &Vec2 {
        &self.0
    }
}

#[expect(clippy::needless_pass_by_value)]
pub fn update_mouse_pos_sy(
    mut mouse_pos: ResMut<MousePos>,
    mut mouse_pos_world: ResMut<MousePosWorld>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let window = window.single();
    let Ok((camera, transform)) = camera.get_single() else {
        return;
    };
    let Some(new_mouse_pos) = window.cursor_position() else {
        return;
    };
    if mouse_pos.0 != new_mouse_pos {
        mouse_pos.0 = new_mouse_pos;
    }

    let Ok(new_mouse_pos_world) = camera.viewport_to_world_2d(transform, new_mouse_pos) else {
        return;
    };
    if mouse_pos_world.0 != new_mouse_pos_world {
        mouse_pos_world.0 = new_mouse_pos_world;
    }
}
