use bevy::{
    input::{
        gestures::{PanGesture, PinchGesture},
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::EguiContexts;

use crate::{
    misc_config::settings::MiscSettings,
    ui::{
        cursor::mouse_pos::MousePos,
        map::{
            settings::TileSettings,
            utils::{get_map_width_height, get_window_width_height},
            window::PointerWithinTilemap,
            zoom::Zoom,
        },
    },
};

#[tracing::instrument(skip_all)]
pub fn mouse_drag_sy(
    buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_origin_pos: Local<Option<MousePos>>,
    mut camera_origin_pos: Local<Option<Vec2>>,
    mouse_pos: Res<MousePos>,
    mut gesture_pan: EventReader<PanGesture>,
    mut camera: Query<(&Camera, &mut Transform)>,
    windows: Query<(Entity, &Window, Option<&PrimaryWindow>)>,
    mut ctx: EguiContexts,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) -> Result {
    if pointer_within_tilemap.is_none() {
        return Ok(());
    }
    let (camera, mut transform) = camera.single_mut()?;

    let pan = gesture_pan.read().map(|a| a.0).sum::<Vec2>();
    transform.translation.x -= pan.x;
    transform.translation.y += pan.y;
    if pan != Vec2::ZERO {
        return Ok(());
    }

    if buttons.pressed(MouseButton::Left) && !ctx.ctx_mut().is_using_pointer() {
        if let Some(origin_pos) = *mouse_origin_pos {
            if !mouse_pos.is_changed() {
                return Ok(());
            }
            let Some(win_wh) = get_window_width_height(&windows, camera) else {
                return Ok(());
            };
            let map_wh = get_map_width_height(camera, &transform);

            let d = map_wh / win_wh * (**mouse_pos - *origin_pos);
            trace!("Mouse moved {d:?} from origin");
            transform.translation.x = camera_origin_pos.unwrap().x - d.x;
            transform.translation.y = camera_origin_pos.unwrap().y + d.y;
        } else {
            *mouse_origin_pos = Some(*mouse_pos.into_inner());
            *camera_origin_pos = Some(transform.translation.truncate());
        }
    } else {
        *mouse_origin_pos = None;
        *camera_origin_pos = None;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn mouse_zoom_sy(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut gesture_pinch: EventReader<PinchGesture>,
    mut gesture_pan: EventReader<PanGesture>,
    mut camera: Query<(&Camera, &GlobalTransform, &mut Projection, &mut Transform)>,
    mut zoom: ResMut<Zoom>,
    mouse_pos: Res<MousePos>,
    tile_settings: Res<TileSettings>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
    misc_settings: Res<MiscSettings>,
) -> Result {
    if pointer_within_tilemap.is_none() {
        return Ok(());
    }
    let (camera, global_transform, mut projection, mut transform) = camera.single_mut()?;
    let Projection::Orthographic(ref mut ort_proj) = *projection else {
        unreachable!();
    };

    let mut u = gesture_pinch.read().map(|a| a.0).sum::<f32>();
    if gesture_pan.is_empty() {
        u += mouse_wheel
            .read()
            .map(|a| match a.unit {
                MouseScrollUnit::Line => a.y * 0.125 * misc_settings.scroll_multiplier_line,
                MouseScrollUnit::Pixel => a.y * 0.0125 * misc_settings.scroll_multiplier_pixel,
            })
            .sum::<f32>();
    }

    if !(1.0 <= (zoom.0 + u)
        && (zoom.0 + u)
            <= f32::from(tile_settings.basemaps[0].max_tile_zoom + misc_settings.additional_zoom))
    {
        return Ok(());
    };

    let orig = transform.translation.xy();
    let orig_scale = ort_proj.scale;
    let Ok(orig_mouse_pos) = camera.viewport_to_world_2d(global_transform, **mouse_pos) else {
        return Ok(());
    };
    zoom.0 += u;
    trace!("Zoom changed from {orig_scale} to {}", zoom.0);

    ort_proj.scale = ((f32::from(tile_settings.basemaps[0].max_tile_zoom) - 1.0) - zoom.0).exp2();

    let d = (orig_mouse_pos - orig) * (ort_proj.scale / orig_scale);
    let Ok(new_mouse_pos) = camera.viewport_to_world_2d(global_transform, **mouse_pos) else {
        return Ok(());
    };
    trace!("View moved by {d:?}");
    transform.translation.x = new_mouse_pos.x - d.x;
    transform.translation.y = new_mouse_pos.y - d.y;
    Ok(())
}
