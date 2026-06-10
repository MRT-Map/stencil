use std::any::Any;

use etcetera::AppStrategy;
use num_traits::real::Real;

use crate::{
    App, impl_load_save, settings,
    settings::{Settings, settings_ui_field},
    utils::{
        coord::{Nnf32, Nnf32UpdateExt, nn},
        file::FOLDERS,
    },
};

settings! {
    #[derive(Eq)] MapSettings {
        init_zoom_as_pc_of_max: Nnf32 = nn(87.5),
        additional_zoom: Nnf32 = nn(4.0),

        max_requests: usize = 0x10000,
        clear_cache_on_startup: bool = false,

        world_screen_ratio: Nnf32 = nn(0.25),

        shortcut_pan_amount: Nnf32 = nn(25.0),
        shortcut_zoom_amount: Nnf32 = nn(0.2),

        invert_scroll: egui::Vec2b = egui::Vec2b::default(),
    }
}
impl_load_save!(toml MapSettings, FOLDERS.in_config_dir("map.toml"), "# Documentation is at https://mrt-map.github.io/stencil3/doc/Map-Settings.html");

impl Settings for MapSettings {
    #[tracing::instrument(skip_all)]
    #[expect(clippy::too_many_lines)]
    fn ui_inner(&mut self, ui: &mut egui::Ui, _tab_state: &mut dyn Any) {
        let default = Self::default();

        settings_ui_field(
            ui,
            &mut self.init_zoom_as_pc_of_max,
            default.init_zoom_as_pc_of_max,
            Some(
                "Zoom level when opening the app, as a percentage of the maximum tile zoom of the basemap.\nFor example, if our basemap has a maximum zoom of 8, setting 87.5% means the app starts with zoom level 87.5% * 8 = 7.",
            ),
            |ui, value| {
                value.update(|mut value| {
                    ui.add(
                        egui::Slider::new(&mut value, 0.0..=200.0)
                            .suffix("%")
                            .text("Initial zoom (as % of max tile zoom)"),
                    );
                    value
                });
            },
        );
        settings_ui_field(
            ui,
            &mut self.additional_zoom,
            default.additional_zoom,
            Some(
                "Increases the maximum zoom so you can zoom in further than the maximum tile zoom",
            ),
            |ui, value| {
                value.update(|mut value| {
                    ui.add(
                        egui::Slider::new(&mut value, 0.0..=10.0).text("Additional zoom levels"),
                    );
                    value
                });
            },
        );

        ui.separator();

        settings_ui_field(
            ui,
            &mut self.clear_cache_on_startup,
            default.clear_cache_on_startup,
            Option::<&str>::None,
            |ui, value| {
                ui.checkbox(value, "Clear tile cache on startup");
            },
        );
        settings_ui_field(
            ui,
            &mut self.max_requests,
            default.max_requests,
            Some("Maximum number of tiles to download at a time"),
            |ui, value| {
                ui.add(egui::Slider::new(value, 1..=0x10000).text("Maximum HTTP GET requests"));
            },
        );

        ui.separator();

        settings_ui_field(
            ui,
            &mut self.world_screen_ratio,
            default.world_screen_ratio,
            Some(
                "Ratio of distance in the world in world units to distance on the screen in pixels at the maximum zoom",
            ),
            |ui, value| {
                value.update(|value| {
                    let (mut world, mut screen) = if value > 1.0 {
                        (value, 1.0)
                    } else {
                        (1.0, 1.0 / value)
                    };
                    let (world_speed, screen_speed) = (world / 32.0, screen / 32.0);
                    ui.add(
                        egui::DragValue::new(&mut world)
                            .range(1.0..=1024.0)
                            .speed(world_speed)
                            .suffix("u"),
                    );
                    ui.label(":");
                    ui.add(
                        egui::DragValue::new(&mut screen)
                            .range(1.0..=1024.0)
                            .speed(screen_speed)
                            .suffix("px"),
                    );
                    ui.label("World : Screen ratio");

                    world / screen
                });
            },
        );

        ui.separator();

        settings_ui_field(
            ui,
            &mut self.shortcut_pan_amount,
            default.shortcut_pan_amount,
            Some("Pixels to move by when any PanMap shortcut is pressed"),
            |ui, value| {
                value.update(|mut value| {
                    ui.add(
                        egui::Slider::new(&mut value, 1.0..=100.0)
                            .suffix("px")
                            .text("Shortcut Pan Amount"),
                    );
                    value
                });
            },
        );

        settings_ui_field(
            ui,
            &mut self.shortcut_zoom_amount,
            default.shortcut_zoom_amount,
            Some("Zoom levels to increase/decrease by when any ZoomMap shortcut is pressed"),
            |ui, value| {
                value.update(|mut value| {
                    ui.add(egui::Slider::new(&mut value, 0.01..=1.0).text("Shortcut Zoom Amount"));
                    value
                });
            },
        );

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.invert_scroll != default.invert_scroll,
                    egui::Button::new("⟲"),
                )
                .on_hover_text(format!("Default: {:?}", default.invert_scroll))
                .clicked()
            {
                self.invert_scroll = default.invert_scroll;
            }

            ui.checkbox(&mut self.invert_scroll.x, "Horizontal");
            ui.checkbox(&mut self.invert_scroll.y, "Vertical");
            ui.label("Inverted scroll");
        });
    }
}

impl MapSettings {
    #[must_use]
    pub fn world_screen_ratio_at_zoom(&self, max_tile_zoom: i8, zoom: Nnf32) -> Nnf32 {
        self.world_screen_ratio * (Nnf32::from(max_tile_zoom) - zoom).exp2()
    }
}
impl App {
    #[must_use]
    pub fn world_screen_ratio_with_current_basemap_at_current_zoom(&self) -> Nnf32 {
        self.settings
            .map
            .world_screen_ratio_at_zoom(self.project.basemap.max_tile_zoom, self.ui.map.zoom)
    }
}
