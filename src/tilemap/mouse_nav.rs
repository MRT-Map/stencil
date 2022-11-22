use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
};
use bevy_mouse_tracking_plugin::{MainCamera, MousePos, MousePosWorld};

use crate::{
    tilemap::{
        utils::{get_map_width_height, get_window_width_height},
        zoom::Zoom,
    },
    ui::HoveringOverGui,
};

#[tracing::instrument(skip_all)]
pub fn mouse_drag_sy(
    buttons: Res<Input<MouseButton>>,
    mut mouse_origin_pos: Local<Option<MousePos>>,
    mut camera_origin_pos: Local<Option<Vec2>>,
    mouse_pos: Res<MousePos>,
    mut camera: Query<(&Camera, &mut GlobalTransform), With<MainCamera>>,
    windows: Res<Windows>,
    hovering_over_gui: Res<HoveringOverGui>,
) {
    if hovering_over_gui.0 {
        return;
    }
    let (camera, mut transform) = camera.single_mut();
    if buttons.pressed(MouseButton::Left) {
        if let Some(origin_pos) = *mouse_origin_pos {
            if !mouse_pos.is_changed() {
                return;
            }
            let win_wh = if let Some(win_wh) = get_window_width_height(&windows, camera) {
                win_wh
            } else {
                return;
            };
            let map_wh = get_map_width_height(camera, &transform);

            let d = map_wh / win_wh * (**mouse_pos - *origin_pos);
            trace!("Mouse moved {d:?} from origin");
            transform.translation_mut().x = camera_origin_pos.unwrap().x - d.x;
            transform.translation_mut().y = camera_origin_pos.unwrap().y - d.y;
        } else {
            *mouse_origin_pos = Some(*mouse_pos.into_inner());
            *camera_origin_pos = Some(transform.translation().truncate());
        }
    } else {
        *mouse_origin_pos = None;
        *camera_origin_pos = None;
    }
}

#[tracing::instrument(skip_all)]
pub fn mouse_zoom_sy(
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera: Query<(&mut OrthographicProjection, &mut GlobalTransform), With<MainCamera>>,
    mut zoom: ResMut<Zoom>,
    hovering_over_gui: Res<HoveringOverGui>,
    mouse_pos_world: Query<&MousePosWorld>,
) {
    if hovering_over_gui.0 {
        return;
    }
    let (mut ort_proj, mut transform) =
        camera.single_mut();
    for ev in scroll_evr.iter() {
        let u = match ev.unit {
            MouseScrollUnit::Line => ev.y * 0.125,
            MouseScrollUnit::Pixel => ev.y * 0.0125,
        };
        if 1.0 <= (zoom.0 + u) && (zoom.0 + u) <= 11.0 {
            let orig = transform.translation().xy();
            let orig_scale = ort_proj.scale;
            let orig_mouse_pos = mouse_pos_world.single();
            zoom.0 += u;
            trace!("Zoom changed from {orig_scale} to {{zoom.0}}");

            ort_proj.scale = 2f32.powf(7.0 - zoom.0);

            let d = (orig_mouse_pos.xy() - orig) * (ort_proj.scale / orig_scale);
            let new_mouse_pos = mouse_pos_world.single();
            trace!("View moved by {d:?}");
            transform.translation_mut().x = new_mouse_pos.x - d.x;
            transform.translation_mut().y = new_mouse_pos.y - d.y;
        }
    }
}
