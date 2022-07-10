use bevy::prelude::*;
use bevy::render::camera::RenderTarget;

pub fn get_window_width_height(
    windows: &Res<Windows>,
    camera: &Camera
) -> Vec2 {
    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    Vec2::new(wnd.width() as f32, wnd.height() as f32)
}

pub fn get_map_width_height(
    camera: &Camera,
    transform: &GlobalTransform
) -> Vec2 {
    let (left, top, right, bottom) = get_map_coords_of_edges(camera, transform);
    Vec2::new(right - left, bottom - top)
}

pub fn get_cursor_world_pos(
    windows: &Res<Windows>,
    camera: &Camera,
    transform: &GlobalTransform
) -> Option<Vec2> {
    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    if let Some(screen_pos) = wnd.cursor_position() {
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = transform.compute_matrix() * camera.projection_matrix.inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        Some(world_pos.truncate())
    } else {None}
}

// https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn get_map_coords_of_edges(
    camera: &Camera,
    transform: &GlobalTransform
) -> (f32, f32, f32, f32) {
    let ndc_tl = Vec2::new(0.0, 0.0) - Vec2::ONE;
    let ndc_br = Vec2::new(2.0, 2.0) - Vec2::ONE;

    let ndc_to_world = transform.compute_matrix() * camera.projection_matrix.inverse();

    let world_pos_tl = ndc_to_world.project_point3(ndc_tl.extend(-1.0));
    let world_pos_br = ndc_to_world.project_point3(ndc_br.extend(-1.0));

    let world_pos_tl: Vec2 = world_pos_tl.truncate();
    let world_pos_br: Vec2 = world_pos_br.truncate();

    (world_pos_tl.x, world_pos_tl.y, world_pos_br.x, world_pos_br.y)
}