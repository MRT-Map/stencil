use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    App,
    map::{basemap::Basemap, settings::MapSettings},
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
    pub prev_cursor_world_pos: Option<geo::Coord<f32>>,
}

impl Default for MapWindow {
    fn default() -> Self {
        Self {
            centre_coord: geo::Coord::<f32>::default(),
            zoom: 0.0,
            prev_cursor_world_pos: None,
        }
    }
}
impl MapWindow {
    pub fn reset(&mut self, app: &App) {
        self.reset2(&app.map_settings, &app.project.basemap);
    }
    pub fn reset2(&mut self, map_settings: &MapSettings, basemap: &Basemap) {
        self.centre_coord = geo::Coord::zero();
        self.zoom = map_settings.init_zoom_as_pc_of_max / 100.0 * f32::from(basemap.max_tile_zoom);
    }
}
impl DockLayout {
    pub fn map_window_mut(&mut self) -> &mut MapWindow {
        let Some((_, DockWindows::MapWindow(map_window))) =
            self.0.iter_all_tabs_mut().find(|(_, a)| a.title() == "Map")
        else {
            unreachable!("Cannot find map window");
        };
        map_window
    }
    pub fn reset_map_window(&mut self, app: &App) {
        self.reset_map_window2(&app.map_settings, &app.project.basemap);
    }
    pub fn reset_map_window2(&mut self, map_settings: &MapSettings, basemap: &Basemap) {
        self.map_window_mut().reset2(map_settings, basemap);
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
        let pt1 = self.world_to_screen(app, response.rect.center(), geo::coord! {x: 0.0, y: 0.0});
        let pt2 = self.world_to_screen(
            app,
            response.rect.center(),
            geo::coord! {x: 100.0, y: 100.0},
        );
        painter.circle_filled(pt1, 5.0, egui::Color32::YELLOW);
        painter.circle_filled(pt2, 5.0, egui::Color32::WHITE);

        if let Some(hover_pos) = response.hover_pos() {
            let mut cursor_world_pos = self.screen_to_world(app, response.rect.center(), hover_pos);

            let old_zoom = self.zoom;
            self.zoom += ui.ctx().input(egui::InputState::zoom_delta).log2();
            self.zoom = self.zoom.clamp(
                0.0,
                f32::from(app.project.basemap.max_tile_zoom) + app.map_settings.additional_zoom,
            );

            if (old_zoom - self.zoom).abs() > f32::EPSILON {
                let new_cursor_world_pos =
                    self.screen_to_world(app, response.rect.center(), hover_pos);
                self.centre_coord = self.centre_coord + cursor_world_pos - new_cursor_world_pos;
                cursor_world_pos = new_cursor_world_pos;
            }

            let mut translation = ui.ctx().input(egui::InputState::translation_delta)
                * app
                    .map_settings
                    .world_screen_ratio_at_zoom(app.project.basemap.max_tile_zoom, self.zoom);
            translation.x -= if ui.ctx().input_mut(|a| {
                a.consume_shortcut(
                    &app.shortcut_settings
                        .action_to_keyboard(ShortcutAction::MoveMapLeft),
                )
            }) {
                25.0
            } else {
                0.0
            };
            translation.x += if ui.ctx().input_mut(|a| {
                a.consume_shortcut(
                    &app.shortcut_settings
                        .action_to_keyboard(ShortcutAction::MoveMapRight),
                )
            }) {
                25.0
            } else {
                0.0
            };
            translation.y += if ui.ctx().input_mut(|a| {
                a.consume_shortcut(
                    &app.shortcut_settings
                        .action_to_keyboard(ShortcutAction::MoveMapDown),
                )
            }) {
                25.0
            } else {
                0.0
            };
            translation.y -= if ui.ctx().input_mut(|a| {
                a.consume_shortcut(
                    &app.shortcut_settings
                        .action_to_keyboard(ShortcutAction::MoveMapUp),
                )
            }) {
                25.0
            } else {
                0.0
            };
            translation += if response.dragged_by(egui::PointerButton::Middle) {
                -response.drag_delta()
                    * app
                        .map_settings
                        .world_screen_ratio_at_zoom(app.project.basemap.max_tile_zoom, self.zoom)
            } else {
                egui::Vec2::ZERO
            };
            self.centre_coord.x += translation.x;
            self.centre_coord.y += translation.y;

            self.prev_cursor_world_pos = Some(cursor_world_pos);
        } else {
            self.prev_cursor_world_pos = None;
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
        self.centre_coord + geo::Coord::from((world_delta.x, world_delta.y))
    }
}
