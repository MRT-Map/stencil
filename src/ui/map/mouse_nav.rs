use std::fmt::{Display, Formatter};

use bevy::{
    input::{
        gestures::PinchGesture,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::EguiContexts;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollMode {
    Pan,
    #[default]
    Zoom,
}
impl Display for ScrollMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Zoom => "Zoom",
                Self::Pan => "Pan",
            }
        )
    }
}

#[tracing::instrument(skip_all)]
pub fn mouse_pan_sy(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut prev_mouse_pos: Local<Option<MousePos>>,
    mouse_pos: Res<MousePos>,
    mut camera: Query<(&Camera, &mut Transform)>,
    windows: Query<(Entity, &Window, Option<&PrimaryWindow>)>,
    mut ctx: EguiContexts,
    tile_settings: Res<TileSettings>,
    misc_settings: Res<MiscSettings>,
    zoom: Res<Zoom>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) -> Result {
    if pointer_within_tilemap.is_none() {
        return Ok(());
    }
    let (camera, mut transform) = camera.single_mut()?;

    let mut d = if misc_settings.scroll_mode == ScrollMode::Pan
        && !keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        let d = mouse_wheel
            .read()
            .map(|a| match a.unit {
                MouseScrollUnit::Line => Vec2::new(a.x, a.y) * misc_settings.scroll_multiplier_line,
                MouseScrollUnit::Pixel => {
                    Vec2::new(a.x, a.y) * 0.01 * misc_settings.scroll_multiplier_pixel
                }
            })
            .sum::<Vec2>()
            * (f32::from(tile_settings.basemaps[0].max_tile_zoom) - zoom.0)
            * tile_settings.basemaps[0].max_zoom_range;
        if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            d.yx()
        } else {
            d
        }
    } else {
        Vec2::ZERO
    };

    d += if buttons.pressed(MouseButton::Left) && !ctx.ctx_mut().is_using_pointer() {
        let prev_mouse_pos = prev_mouse_pos.get_or_insert(*mouse_pos);
        let delta = **mouse_pos - **prev_mouse_pos;
        *prev_mouse_pos = *mouse_pos;
        delta
    } else {
        *prev_mouse_pos = None;
        Vec2::ZERO
    };

    let Some(win_wh) = get_window_width_height(&windows, camera) else {
        return Ok(());
    };
    let map_wh = get_map_width_height(camera, &transform);

    let d = map_wh / win_wh * d;
    trace!("Mouse moved {d:?} from origin");
    transform.translation.x -= d.x;
    transform.translation.y += d.y;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn mouse_zoom_sy(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut gesture_pinch: EventReader<PinchGesture>,
    mut camera: Query<(&Camera, &GlobalTransform, &mut Projection, &mut Transform)>,
    mut zoom: ResMut<Zoom>,
    mouse_pos: Res<MousePos>,
    tile_settings: Res<TileSettings>,
    misc_settings: Res<MiscSettings>,
    pointer_within_tilemap: Option<Res<PointerWithinTilemap>>,
) -> Result {
    if pointer_within_tilemap.is_none() {
        return Ok(());
    }
    let (camera, global_transform, mut projection, mut transform) = camera.single_mut()?;
    let Projection::Orthographic(ref mut ort_proj) = *projection else {
        unreachable!();
    };

    let mut u = 1.5 * gesture_pinch.read().map(|a| a.0).sum::<f32>();
    if misc_settings.scroll_mode == ScrollMode::Zoom
        || keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
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
    }

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
