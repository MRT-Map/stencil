use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    App,
    map::tile_coord::{TILE_CACHE, TextureIdResult, TileCoord},
    mode::EditorMode,
    pointer::ResponsePointerExt,
    project::SkinStatus,
    shortcut::ShortcutAction,
    ui::dock::DockWindow,
};

pub mod basemap;
pub mod context_menu;
pub mod settings;
pub mod state;
pub mod tile_coord;
pub mod toolbar;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct MapWindow;

impl DockWindow for MapWindow {
    fn title(self) -> String {
        "Map".into()
    }
    fn allowed_in_windows(self) -> bool {
        false
    }
    fn is_closeable(self) -> bool {
        false
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        Self::toolbar(app, ui);

        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::all());

        Self::tiles(app, ui, &response, &painter);
        Self::interaction(app, ui, &response);
        Self::components(app, ui, &response, &painter);
        Self::cursor(app, ui, &response, &painter);
    }
}
impl MapWindow {
    fn tiles(app: &App, ui: &egui::Ui, response: &egui::Response, painter: &egui::Painter) {
        let world_boundaries = app.map_world_boundaries(response.rect);
        let tile_zoom = app.project.basemap.tile_zoom(app.ui.map.zoom);
        let tile_screen_size = app
            .project
            .basemap
            .tile_screen_size(&app.map_settings, app.ui.map.zoom);
        let min_tile_coord =
            TileCoord::at_world_coord(world_boundaries.min(), tile_zoom, &app.project.basemap);
        let max_tile_coord =
            TileCoord::at_world_coord(world_boundaries.max(), tile_zoom, &app.project.basemap);
        let min_tile_screen_top_left = app.map_world_to_screen(
            response.rect.center(),
            min_tile_coord.world_top_left(&app.project.basemap),
        );
        let mut tile_screen_top_left = min_tile_screen_top_left;

        let Ok(mut tile_cache) = TILE_CACHE.lock().inspect_err(|e| error!("{e:?}")) else {
            return;
        };

        for tx in min_tile_coord.x..=max_tile_coord.x {
            for ty in min_tile_coord.y..=max_tile_coord.y {
                match TileCoord::new(tile_zoom, tx, ty).texture_id(
                    ui.ctx(),
                    &app.project.basemap,
                    &mut tile_cache,
                ) {
                    Some(TextureIdResult::Success(texture_id)) => {
                        painter.image(
                            texture_id,
                            egui::Rect::from_min_size(
                                tile_screen_top_left,
                                egui::Vec2::splat(tile_screen_size),
                            ),
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    }
                    Some(TextureIdResult::Loading) => {
                        let centre =
                            tile_screen_top_left + egui::Vec2::splat(tile_screen_size / 2.0);
                        let u = tile_screen_size / 32.0;
                        painter.line(
                            vec![
                                centre + egui::vec2(-u, -2.0 * u),
                                centre + egui::vec2(-u, -u),
                                centre + egui::vec2(u, u),
                                centre + egui::vec2(u, 2.0 * u),
                                centre + egui::vec2(-u, 2.0 * u),
                                centre + egui::vec2(-u, u),
                                centre + egui::vec2(u, -u),
                                centre + egui::vec2(u, -2.0 * u),
                                centre + egui::vec2(-u, -2.0 * u),
                            ],
                            egui::epaint::PathStroke::new(
                                tile_screen_size / 48.0,
                                egui::Color32::DARK_GRAY,
                            ),
                        );
                    }
                    None => {}
                }
                tile_screen_top_left.y += tile_screen_size;
            }
            tile_screen_top_left.x += tile_screen_size;
            tile_screen_top_left.y = min_tile_screen_top_left.y;
        }
    }
    fn cursor(app: &App, ui: &egui::Ui, response: &egui::Response, painter: &egui::Painter) {
        if response.hover_pos().is_none() {
            return;
        }
        if response.dragged_by2(egui::PointerButton::Middle) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            return;
        }
        match app.mode {
            EditorMode::Select | EditorMode::Nodes => {
                if app.ui.map.hovered_component.is_some() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                } else {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }
            }
            EditorMode::CreateArea | EditorMode::CreateLine | EditorMode::CreatePoint => {
                let tooltip = |text: &str| {
                    egui::Tooltip::always_open(
                        ui.ctx().to_owned(),
                        response.layer_id,
                        response.id,
                        egui::PopupAnchor::Pointer,
                    )
                    .show(|ui| ui.label(text));
                };
                if app.project.new_component_ns.is_empty() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
                    tooltip("Set a namespace in the toolbar first");
                    return;
                }
                if matches!(app.project.skin_status, SkinStatus::Failed(_)) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
                    tooltip("Skin failed to load. See Project Editor");
                    return;
                }
                if app.project.skin().is_none() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Wait);
                    tooltip("Waiting for skin to load...");
                    return;
                }

                let Some(pointer_screen_pos) = ui.ctx().pointer_latest_pos() else {
                    return;
                };
                let pointer_world_pos =
                    app.map_screen_to_world(response.rect.center(), pointer_screen_pos);
                let crosshair_screen_pos = app.map_world_to_screen(
                    response.rect.center(),
                    geo::coord! { x: pointer_world_pos.x.round(), y: pointer_world_pos.y.round() },
                );
                let (x, y) = (crosshair_screen_pos.x, crosshair_screen_pos.y);

                ui.ctx().set_cursor_icon(egui::CursorIcon::None);
                painter.hline(
                    egui::Rangef::new(x + 1.0 - 16.0, x + 1.0 + 16.0),
                    y + 1.0,
                    egui::Stroke::new(8.0, egui::Color32::BLACK.gamma_multiply(0.25)),
                );
                painter.vline(
                    x + 1.0,
                    egui::Rangef::new(y + 1.0 - 16.0, y + 1.0 + 16.0),
                    egui::Stroke::new(8.0, egui::Color32::BLACK.gamma_multiply(0.25)),
                );
                painter.hline(
                    egui::Rangef::new(x - 16.0, x + 16.0),
                    y,
                    egui::Stroke::new(6.0, egui::Color32::BLACK),
                );
                painter.vline(
                    x,
                    egui::Rangef::new(y - 16.0, y + 16.0),
                    egui::Stroke::new(6.0, egui::Color32::BLACK),
                );
                painter.hline(
                    egui::Rangef::new(x - 14.0, x + 14.0),
                    y,
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
                painter.vline(
                    x,
                    egui::Rangef::new(y - 14.0, y + 14.0),
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
            }
        }
    }
    fn interaction(app: &mut App, ui: &egui::Ui, response: &egui::Response) {
        let Some(hover_pos) = response.hover_pos().or_else(|| {
            response
                .ctx
                .data_mut(|a| {
                    *a.get_temp_mut_or_default::<bool>(Self::HOVERED_OVER_CTX_MENU.into())
                })
                .then(|| ui.ctx().pointer_latest_pos())
                .flatten()
        }) else {
            app.ui.map.cursor_world_pos = None;
            return;
        };
        let mut cursor_world_pos = app.map_screen_to_world(response.rect.center(), hover_pos);

        let old_zoom = app.ui.map.zoom;
        app.ui.map.zoom += ui.ctx().input(egui::InputState::zoom_delta).log2();

        app.ui.map.zoom = app.ui.map.zoom.clamp(
            0.0,
            f32::from(app.project.basemap.max_tile_zoom) + app.map_settings.additional_zoom,
        );

        if (old_zoom - app.ui.map.zoom).abs() > f32::EPSILON {
            let new_cursor_world_pos = app.map_screen_to_world(response.rect.center(), hover_pos);
            app.ui.map.centre_coord =
                app.ui.map.centre_coord + cursor_world_pos - new_cursor_world_pos;
            cursor_world_pos = new_cursor_world_pos;
        }

        for (action, sign) in [
            (ShortcutAction::ZoomMapOut, -1.0),
            (ShortcutAction::ZoomMapIn, 1.0),
        ] {
            app.ui.map.zoom += if ui.ctx().input_mut(|a| {
                a.consume_shortcut(&app.shortcut_settings.action_to_shortcut(action))
            }) {
                app.map_settings.shortcut_zoom_amount * sign
            } else {
                0.0
            }
        }

        let world_screen_ratio = app.world_screen_ratio_with_current_basemap_at_current_zoom();

        let invert = app.map_settings.invert_scroll;
        let mut translation = ui.ctx().input(egui::InputState::translation_delta)
            * world_screen_ratio
            * egui::vec2(
                if invert.x { -1.0 } else { 1.0 },
                if invert.y { -1.0 } else { 1.0 },
            );

        for (is_x, action, sign) in [
            (true, ShortcutAction::PanMapLeft, -1.0),
            (true, ShortcutAction::PanMapRight, 1.0),
            (false, ShortcutAction::PanMapDown, 1.0),
            (false, ShortcutAction::PanMapUp, -1.0),
        ] {
            *(if is_x {
                &mut translation.x
            } else {
                &mut translation.y
            }) += if ui.ctx().input_mut(|a| {
                a.consume_shortcut(&app.shortcut_settings.action_to_shortcut(action))
            }) {
                app.map_settings.shortcut_pan_amount * sign * world_screen_ratio
            } else {
                0.0
            };
        }
        translation += if response.dragged_by2(egui::PointerButton::Middle) {
            -response.drag_delta() * world_screen_ratio
        } else {
            egui::Vec2::ZERO
        };
        app.ui.map.centre_coord.x += translation.x;
        app.ui.map.centre_coord.y += translation.y;

        app.ui.map.cursor_world_pos = Some(cursor_world_pos);
    }
    fn components(
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        Self::paint_components(app, ui, response, painter);
        Self::select_hovered_component(app, ui, response, painter);
        Self::component_context_menu(app, response);
        Self::move_components(app, response);

        match app.mode {
            EditorMode::CreatePoint => Self::create_point(app, ui, response, painter),
            EditorMode::CreateLine => Self::create_line(app, ui, response, painter),
            EditorMode::CreateArea => Self::create_area(app, ui, response, painter),
            _ => {}
        }
    }
}
