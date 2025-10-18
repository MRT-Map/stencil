use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::{App, file::data_dir, impl_load_save, settings::Settings, settings_field};

settings_field!(
    MapSettings,
    init_zoom_as_pc_of_max_is_default,
    init_zoom_as_pc_of_max,
    f32
);
settings_field!(
    MapSettings,
    additional_zoom_is_default,
    additional_zoom,
    f32
);
settings_field!(MapSettings, max_requests_is_default, max_requests, usize);
settings_field!(
    MapSettings,
    clear_cache_on_startup_is_default,
    clear_cache_on_startup,
    bool
);
settings_field!(
    MapSettings,
    world_screen_ratio_is_default,
    world_screen_ratio,
    f32
);
settings_field!(
    MapSettings,
    shortcut_pan_amount_is_default,
    shortcut_pan_amount,
    f32
);
settings_field!(
    MapSettings,
    shortcut_zoom_amount_is_default,
    shortcut_zoom_amount,
    f32
);

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct MapSettings {
    #[serde(skip_serializing_if = "init_zoom_as_pc_of_max_is_default")]
    pub init_zoom_as_pc_of_max: f32,
    #[serde(skip_serializing_if = "additional_zoom_is_default")]
    pub additional_zoom: f32,

    #[serde(skip_serializing_if = "max_requests_is_default")]
    pub max_requests: usize,
    #[serde(skip_serializing_if = "clear_cache_on_startup_is_default")]
    pub clear_cache_on_startup: bool,

    #[serde(skip_serializing_if = "world_screen_ratio_is_default")]
    pub world_screen_ratio: f32,

    #[serde(skip_serializing_if = "shortcut_pan_amount_is_default")]
    pub shortcut_pan_amount: f32,
    #[serde(skip_serializing_if = "shortcut_zoom_amount_is_default")]
    pub shortcut_zoom_amount: f32,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            init_zoom_as_pc_of_max: 87.5,
            additional_zoom: 4.0,
            max_requests: 0x10000,
            clear_cache_on_startup: false,
            world_screen_ratio: 0.25,
            shortcut_pan_amount: 25.0,
            shortcut_zoom_amount: 0.2,
        }
    }
}

impl_load_save!(toml MapSettings, data_dir("settings").join("map.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#map_settingstoml");

impl Settings for MapSettings {
    fn ui_inner(&mut self, ui: &mut egui::Ui, _tab_state: &mut dyn Any) {
        self.ui_field(
            ui,
            |a| a.init_zoom_as_pc_of_max,
            |a| &a.init_zoom_as_pc_of_max,
            |a| &mut a.init_zoom_as_pc_of_max,
            Some("Zoom level when opening the app, as a percentage of the maximum tile zoom of the basemap.\nFor example, if our basemap has a maximum zoom of 8, setting 87.5% means the app starts with zoom level 87.5% * 8 = 7."),
            |ui, value| {ui.add(egui::Slider::new(value, 0.0..=200.0).suffix("%").text("Initial zoom (as % of max tile zoom)"));}
        );
        self.ui_field(
            ui,
            |a| a.additional_zoom,
            |a| &a.additional_zoom,
            |a| &mut a.additional_zoom,
            Some(
                "Increases the maximum zoom so you can zoom in further than the maximum tile zoom",
            ),
            |ui, value| {
                ui.add(egui::Slider::new(value, 0.0..=10.0).text("Additional zoom levels"));
            },
        );

        ui.separator();

        self.ui_field(
            ui,
            |a| a.clear_cache_on_startup,
            |a| &a.clear_cache_on_startup,
            |a| &mut a.clear_cache_on_startup,
            Option::<&str>::None,
            |ui, value| {
                ui.checkbox(value, "Clear tile cache on startup");
            },
        );
        self.ui_field(
            ui,
            |a| a.max_requests,
            |a| &a.max_requests,
            |a| &mut a.max_requests,
            Some("Maximum number of tiles to download at a time"),
            |ui, value| {
                ui.add(egui::Slider::new(value, 1..=0x10000).text("Maximum HTTP GET requests"));
            },
        );

        ui.separator();

        self.ui_field(
            ui,
            |a| a.world_screen_ratio,
            |a| &a.world_screen_ratio,
            |a| &mut a.world_screen_ratio,
            Some("Ratio of distance in the world in world units to distance on the screen in pixels at the maximum zoom"),
            |ui, value| {
                let (mut world, mut screen) = if *value > 1.0 {
                    (*value, 1.0)
                } else {
                    (1.0, 1.0 / *value)
                };
                let (world_speed, screen_speed) = (world / 32.0, screen / 32.0);
                ui.add(egui::DragValue::new(&mut world).range(1.0..=1024.0).speed(world_speed).suffix("u"));
                ui.label(":");
                ui.add(egui::DragValue::new(&mut screen).range(1.0..=1024.0).speed(screen_speed).suffix("px"));
                ui.label("World : Screen ratio");

                *value = world / screen;
            }
        );

        ui.separator();

        self.ui_field(
            ui,
            |a| a.shortcut_pan_amount,
            |a| &a.shortcut_pan_amount,
            |a| &mut a.shortcut_pan_amount,
            Some("Pixels to move by when any PanMap shortcut is pressed"),
            |ui, value| {
                ui.add(
                    egui::Slider::new(value, 1.0..=100.0)
                        .suffix("px")
                        .text("Shortcut Pan Amount"),
                );
            },
        );

        self.ui_field(
            ui,
            |a| a.shortcut_zoom_amount,
            |a| &a.shortcut_zoom_amount,
            |a| &mut a.shortcut_zoom_amount,
            Some("Zoom levels to increase/decrease by when any ZoomMap shortcut is pressed"),
            |ui, value| {
                ui.add(egui::Slider::new(value, 0.01..=1.0).text("Shortcut Zoom Amount"));
            },
        );
    }
}

impl MapSettings {
    pub fn world_screen_ratio_at_zoom(&self, max_tile_zoom: i8, zoom: f32) -> f32 {
        self.world_screen_ratio * (f32::from(max_tile_zoom) - zoom).exp2()
    }
}
impl App {
    pub fn world_screen_ratio_with_current_basemap_at_zoom(&self, zoom: f32) -> f32 {
        self.map_settings
            .world_screen_ratio_at_zoom(self.project.basemap.max_tile_zoom, zoom)
    }
}
