use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App,
    map::{MapWindow, basemap::Basemap, settings::MapSettings},
    project::{
        component_list::ComponentList,
        pla3::{FullId, PlaComponent, PlaNode},
        skin::SkinType,
    },
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MapState {
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

impl Default for MapState {
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
impl App {
    pub fn map_reset_view(&mut self) {
        self.ui
            .map
            .reset_view(&self.map_settings, &self.project.basemap);
    }
    pub fn map_world_to_screen(
        &self,
        map_centre: egui::Pos2,
        world: geo::Coord<f32>,
    ) -> egui::Pos2 {
        self.ui
            .map
            .world_to_screen(&self.map_settings, &self.project.basemap, map_centre, world)
    }
    pub fn map_screen_to_world(
        &self,
        map_centre: egui::Pos2,
        screen: egui::Pos2,
    ) -> geo::Coord<f32> {
        self.ui.map.screen_to_world(
            &self.map_settings,
            &self.project.basemap,
            map_centre,
            screen,
        )
    }
    pub fn map_world_boundaries(&self, map_rect: egui::Rect) -> geo::Rect<f32> {
        self.ui
            .map
            .map_world_boundaries(&self.map_settings, &self.project.basemap, map_rect)
    }
    pub fn map_selected_components(&self) -> Vec<&PlaComponent> {
        self.ui.map.selected_components(&self.project.components)
    }
    pub fn map_selected_components_mut(&mut self) -> Vec<&mut PlaComponent> {
        self.ui
            .map
            .selected_components_mut(&mut self.project.components)
    }
    pub fn map_zoom_level(&self) -> u8 {
        self.ui.map.zoom_level(&self.project.basemap)
    }
}
impl MapState {
    pub fn reset_view(&mut self, map_settings: &MapSettings, basemap: &Basemap) {
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
        self.centre_coord + geo::coord! { x: world_delta.x, y: world_delta.y }
    }

    pub fn map_world_boundaries(
        &self,
        map_settings: &MapSettings,
        basemap: &Basemap,
        map_rect: egui::Rect,
    ) -> geo::Rect<f32> {
        geo::Rect::new(
            self.screen_to_world(map_settings, basemap, map_rect.center(), map_rect.min),
            self.screen_to_world(map_settings, basemap, map_rect.center(), map_rect.max),
        )
    }

    pub fn zoom_level(&self, basemap: &Basemap) -> u8 {
        (basemap.max_tile_zoom - self.zoom.round() as i8)
            .max(0)
            .cast_unsigned()
    }
}
