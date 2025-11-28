use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    App,
    map::{
        basemap::Basemap,
        settings::MapSettings,
        tile_coord::{TILE_CACHE, TextureIdResult, TileCoord},
    },
    mode::EditorMode,
    project::{
        SkinStatus,
        component_list::ComponentList,
        pla3::{FullId, PlaComponent, PlaNode},
        skin::SkinType,
    },
    shortcut::ShortcutAction,
    ui::dock::{DockLayout, DockWindow, DockWindows},
};

pub mod basemap;
pub mod settings;
pub mod tile_coord;
pub mod toolbar;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MapWindow {
    pub centre_coord: geo::Coord<f32>,
    pub zoom: f32,
    pub cursor_world_pos: Option<geo::Coord<f32>>,

    #[serde(skip)]
    pub created_nodes: Vec<PlaNode>,
    #[serde(skip)]
    pub created_point_type: Option<Arc<SkinType>>,
    #[serde(skip)]
    pub created_line_type: Option<Arc<SkinType>>,
    #[serde(skip)]
    pub created_area_type: Option<Arc<SkinType>>,

    #[serde(skip)]
    pub hovered_component: Option<FullId>,
    #[serde(skip)]
    pub selected_components: Vec<FullId>,
    #[serde(skip)]
    pub clipboard: Vec<PlaComponent>,
}

impl Default for MapWindow {
    fn default() -> Self {
        Self {
            centre_coord: geo::Coord::<f32>::default(),
            zoom: 0.0,
            cursor_world_pos: None,
            created_nodes: Vec::new(),
            created_point_type: None,
            created_line_type: None,
            created_area_type: None,
            hovered_component: None,
            selected_components: Vec::new(),
            clipboard: Vec::new(),
        }
    }
}
impl MapWindow {
    pub fn reset(&mut self, app: &App) {
        self.reset2(&app.map_settings, &app.project.basemap);
    }
    pub fn reset2(&mut self, map_settings: &MapSettings, basemap: &Basemap) {
        info!("Resetting map view");
        self.centre_coord = geo::Coord::zero();
        self.zoom = map_settings.init_zoom_as_pc_of_max / 100.0 * f32::from(basemap.max_tile_zoom);
    }

    pub fn selected_components<'a>(
        &self,
        component_list: &'a ComponentList,
    ) -> Vec<&'a PlaComponent> {
        if self.selected_components.is_empty() {
            return Vec::new();
        }
        component_list
            .iter()
            .filter(|a| self.selected_components.contains(&a.full_id))
            .collect::<Vec<_>>()
    }
    pub fn selected_components_mut<'a>(
        &self,
        component_list: &'a mut ComponentList,
    ) -> Vec<&'a mut PlaComponent> {
        if self.selected_components.is_empty() {
            return Vec::new();
        }
        component_list
            .iter_mut()
            .filter(|a| self.selected_components.contains(&a.full_id))
            .collect::<Vec<_>>()
    }
}

impl DockLayout {
    pub fn map_window(&self) -> &MapWindow {
        let Some((_, DockWindows::MapWindow(map_window))) =
            self.0.iter_all_tabs().find(|(_, a)| a.title() == "Map")
        else {
            unreachable!("Cannot find map window");
        };
        map_window
    }
    pub fn map_window_mut(&mut self) -> &mut MapWindow {
        let Some((_, DockWindows::MapWindow(map_window))) =
            self.0.iter_all_tabs_mut().find(|(_, a)| a.title() == "Map")
        else {
            unreachable!("Cannot find map window");
        };
        map_window
    }
}
impl App {
    pub fn reset_map_window(&mut self) {
        self.ui
            .dock_layout
            .map_window_mut()
            .reset2(&self.map_settings, &self.project.basemap);
    }
}

impl DockWindow for MapWindow {
    fn title(&self) -> String {
        "Map".into()
    }
    fn allowed_in_windows(&self) -> bool {
        false
    }
    fn is_closeable(&self) -> bool {
        false
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        self.toolbar(app, ui);

        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::all());

        self.tiles(app, ui, &response, &painter);
        self.interaction(app, ui, &response);
        self.components(app, ui, &response, &painter);
        self.cursor(app, ui, &response, &painter);
    }
}
impl MapWindow {
    fn tiles(&self, app: &App, ui: &egui::Ui, response: &egui::Response, painter: &egui::Painter) {
        let world_boundaries = self.map_world_boundaries(app, response.rect);
        let tile_zoom = app.project.basemap.tile_zoom(self.zoom);
        let tile_screen_size = app
            .project
            .basemap
            .tile_screen_size(&app.map_settings, self.zoom);
        let min_tile_coord =
            TileCoord::at_world_coord(world_boundaries.min(), tile_zoom, &app.project.basemap);
        let max_tile_coord =
            TileCoord::at_world_coord(world_boundaries.max(), tile_zoom, &app.project.basemap);
        let min_tile_screen_top_left = self.world_to_screen(
            app,
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
    fn cursor(&self, app: &App, ui: &egui::Ui, response: &egui::Response, painter: &egui::Painter) {
        if response.hover_pos().is_none() {
            return;
        }
        if response.dragged_by(egui::PointerButton::Middle) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            return;
        }
        match app.mode {
            EditorMode::Select | EditorMode::Nodes => {
                if self.hovered_component.is_some() {
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
                    self.screen_to_world(app, response.rect.center(), pointer_screen_pos);
                let crosshair_screen_pos = self.world_to_screen(
                    app,
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
    fn interaction(&mut self, app: &mut App, ui: &egui::Ui, response: &egui::Response) {
        let Some(hover_pos) = response.hover_pos() else {
            self.cursor_world_pos = None;
            return;
        };
        let mut cursor_world_pos = self.screen_to_world(app, response.rect.center(), hover_pos);

        let old_zoom = self.zoom;
        self.zoom += ui.ctx().input(egui::InputState::zoom_delta).log2();

        self.zoom = self.zoom.clamp(
            0.0,
            f32::from(app.project.basemap.max_tile_zoom) + app.map_settings.additional_zoom,
        );

        if (old_zoom - self.zoom).abs() > f32::EPSILON {
            let new_cursor_world_pos = self.screen_to_world(app, response.rect.center(), hover_pos);
            self.centre_coord = self.centre_coord + cursor_world_pos - new_cursor_world_pos;
            cursor_world_pos = new_cursor_world_pos;
        }

        for (action, sign) in [
            (ShortcutAction::ZoomMapOut, -1.0),
            (ShortcutAction::ZoomMapIn, 1.0),
        ] {
            self.zoom += if ui.ctx().input_mut(|a| {
                a.consume_shortcut(&app.shortcut_settings.action_to_shortcut(action))
            }) {
                app.map_settings.shortcut_zoom_amount * sign
            } else {
                0.0
            }
        }

        let world_screen_ratio = app.world_screen_ratio_with_current_basemap_at_zoom(self.zoom);

        let invert = app.map_settings.invert_scroll;
        let mut translation = ui.ctx().input(egui::InputState::translation_delta)
            * world_screen_ratio
            * egui::Vec2::new(
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
        translation += if response.dragged_by(egui::PointerButton::Middle) {
            -response.drag_delta() * world_screen_ratio
        } else {
            egui::Vec2::ZERO
        };
        self.centre_coord.x += translation.x;
        self.centre_coord.y += translation.y;

        self.cursor_world_pos = Some(cursor_world_pos);
    }
    fn components(
        &mut self,
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        self.paint_components(app, ui, response, painter);
        self.select_components(app, ui, response);
        self.move_components(app, response);

        match app.mode {
            EditorMode::CreatePoint => self.create_point(app, ui, response, painter),
            EditorMode::CreateLine => self.create_line(app, ui, response, painter),
            EditorMode::CreateArea => self.create_area(app, ui, response, painter),
            _ => {}
        }
    }
}

impl MapWindow {
    pub fn world_to_screen(
        &self,
        app: &App,
        map_centre: egui::Pos2,
        world: geo::Coord<f32>,
    ) -> egui::Pos2 {
        self.world_to_screen2(&app.map_settings, &app.project.basemap, map_centre, world)
    }
    pub fn world_to_screen2(
        &self,
        map_settings: &MapSettings,
        basemap: &Basemap,
        map_centre: egui::Pos2,
        world: geo::Coord<f32>,
    ) -> egui::Pos2 {
        let world_delta = world - self.centre_coord;
        let screen_delta =
            world_delta / map_settings.world_screen_ratio_at_zoom(basemap.max_tile_zoom, self.zoom);
        map_centre + egui::Vec2::from(screen_delta.x_y())
    }

    pub fn screen_to_world(
        &self,
        app: &App,
        map_centre: egui::Pos2,
        screen: egui::Pos2,
    ) -> geo::Coord<f32> {
        self.screen_to_world2(&app.map_settings, &app.project.basemap, map_centre, screen)
    }
    pub fn screen_to_world2(
        &self,
        map_settings: &MapSettings,
        basemap: &Basemap,
        map_centre: egui::Pos2,
        screen: egui::Pos2,
    ) -> geo::Coord<f32> {
        let screen_delta = screen - map_centre;
        let world_delta = screen_delta
            * map_settings.world_screen_ratio_at_zoom(basemap.max_tile_zoom, self.zoom);
        self.centre_coord + geo::coord! { x: world_delta.x, y: world_delta.y }
    }

    pub fn map_world_boundaries(&self, app: &App, map_rect: egui::Rect) -> geo::Rect<f32> {
        geo::Rect::new(
            self.screen_to_world(app, map_rect.center(), map_rect.min),
            self.screen_to_world(app, map_rect.center(), map_rect.max),
        )
    }

    pub fn zoom_level(&self, app: &App) -> u8 {
        (app.project.basemap.max_tile_zoom - self.zoom.round() as i8)
            .max(0)
            .cast_unsigned()
    }
}
