use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    App,
    map::{basemap::Basemap, settings::MapSettings},
    ui::dock::{DockLayout, DockWindow, DockWindows},
};

pub mod basemap;
pub mod settings;
pub mod tile_coord;
pub mod toolbar;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MapWindow {
    centre_coord: geo::Coord<f32>,
    zoom: f32,
    prev_cursor_world_pos: Option<geo::Coord<f32>>,
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
    pub fn reset(&mut self, map_settings: &MapSettings, basemap: &Basemap) {
        self.centre_coord = geo::Coord::zero();
        self.zoom = map_settings.init_zoom_as_pc_of_max / 100.0 * f32::from(basemap.max_tile_zoom);
    }
}
impl DockLayout {
    pub fn reset_map_window(&mut self, map_settings: &MapSettings, basemap: &Basemap) {
        let Some((_, DockWindows::MapWindow(map_window))) =
            self.0.iter_all_tabs_mut().find(|(_, a)| a.title() == "Map")
        else {
            error!("Cannot find map window");
            return;
        };
        map_window.reset(map_settings, basemap);
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
        let pt1 = self.world_to_screen(
            &app.map_settings,
            &Basemap::default(),
            response.rect.center(),
            geo::coord! {x: 0.0, y: 0.0},
        );
        let pt2 = self.world_to_screen(
            &app.map_settings,
            &Basemap::default(),
            response.rect.center(),
            geo::coord! {x: 100.0, y: 100.0},
        );
        painter.circle_filled(pt1, 5.0, egui::Color32::YELLOW);
        painter.circle_filled(pt2, 5.0, egui::Color32::WHITE);

        if let Some(hover_pos) = response.hover_pos() {
            self.prev_cursor_world_pos = Some(self.screen_to_world(
                &app.map_settings,
                &Basemap::default(),
                response.rect.center(),
                hover_pos,
            ));
            self.zoom += ui.ctx().input(|a| a.smooth_scroll_delta.y / 100.0);
            self.zoom = self.zoom.clamp(
                0.0,
                Basemap::default().max_tile_zoom as f32 + app.map_settings.additional_zoom,
            );
        } else {
            self.prev_cursor_world_pos = None;
        }
    }
}

impl MapWindow {
    pub fn world_to_screen(
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
