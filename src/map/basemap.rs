use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    App, URL_REPLACER,
    file::{cache_dir, safe_delete},
    map::{settings::MapSettings, tile_coord::TileCoord},
    settings::settings_ui_field,
    ui::notif::NotifState,
};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
pub enum BasemapURLType {
    DynMap,
    Regular,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Basemap {
    pub url: String,
    pub extension: String,
    pub max_tile_zoom: i8,
    pub max_zoom_world_size: f32,
    pub url_type: BasemapURLType,
    pub offset: (f32, f32),
}

impl Default for Basemap {
    fn default() -> Self {
        Self {
            url: "https://dynmap.minecartrapidtransit.net/main/tiles/new/flat".into(),
            extension: "png".into(),
            max_tile_zoom: 8,
            max_zoom_world_size: 32.0,
            url_type: BasemapURLType::DynMap,
            offset: (0.5, 32.5),
        }
    }
}

impl Basemap {
    #[must_use]
    pub fn url(&self, tile_coord: TileCoord) -> String {
        match self.url_type {
            BasemapURLType::DynMap => {
                let z = 2.0f32.powi(i32::from(self.max_tile_zoom - tile_coord.z));
                let xy = egui::vec2(tile_coord.x as f32, -tile_coord.y as f32);

                let group = (xy * z / self.max_zoom_world_size).floor();

                let num_in_group = xy * z;

                let mut zzz = String::new();
                let mut i = self.max_tile_zoom;
                while i > tile_coord.z {
                    zzz += "z";
                    i -= 1;
                }

                if !zzz.is_empty() {
                    zzz += "_";
                }

                format!(
                    "{}/{}_{}/{zzz}{}_{}.{}",
                    self.url,
                    group.x as i32,
                    group.y as i32,
                    num_in_group.x as i32,
                    num_in_group.y as i32,
                    self.extension
                )
            }
            BasemapURLType::Regular => {
                format!(
                    "{}/{}/{}/{}.{}",
                    self.url, tile_coord.z, tile_coord.x, tile_coord.y, self.extension
                )
            }
        }
    }
    #[must_use]
    pub fn cache_path(&self) -> PathBuf {
        cache_dir("tile-cache").join(URL_REPLACER.replace_all(&self.url, "").as_ref())
    }
    pub fn clear_cache_path(&self, notifs: &mut NotifState) {
        let _ = safe_delete(self.cache_path(), notifs);
    }
}

impl Basemap {
    pub fn tile_zoom(&self, zoom: f32) -> i8 {
        (zoom.round() as i8).clamp(0, self.max_tile_zoom)
    }
    pub fn tile_world_size(&self, zoom: i8) -> f32 {
        self.max_zoom_world_size * 2.0f32.powi(i32::from(self.max_tile_zoom - zoom))
    }
    pub fn tile_screen_size(&self, map_settings: &MapSettings, zoom: f32) -> f32 {
        self.tile_world_size(self.tile_zoom(zoom))
            / map_settings.world_screen_ratio_at_zoom(self.max_tile_zoom, zoom)
    }
}

impl Basemap {
    pub fn config_ui(&mut self, ui: &mut egui::Ui) {
        let default = Self::default();

        settings_ui_field(
            ui,
            &mut self.url_type,
            default.url_type,
            Option::<&str>::None,
            |ui, value| {
                ui.label("Tile URL Type");
                ui.selectable_value(value, BasemapURLType::Regular, "Regular");
                ui.selectable_value(value, BasemapURLType::DynMap, "DynMap");
            },
        );
        ui.label(match self.url_type {
            BasemapURLType::Regular => "Format: <base url>/z/y/x.<extension>",
            BasemapURLType::DynMap => "Format: <base url>/gx_gy/(zzz.._)x_y.<extension>",
        });

        settings_ui_field(
            ui,
            &mut self.url,
            default.url,
            Some("The base URL of the tile source, with no suffixing slash"),
            |ui, value| {
                ui.add(egui::TextEdit::singleline(value).desired_width(200.0));
                ui.label("Base URL");
            },
        );
        if let Err(e) = surf::Url::parse(&self.url) {
            ui.colored_label(egui::Color32::RED, format!("Invalid URL: {e:?}"));
        }

        settings_ui_field(
            ui,
            &mut self.extension,
            default.extension,
            Some("The extension/format of the tile images"),
            |ui, value| {
                ui.add(egui::TextEdit::singleline(value).desired_width(50.0));
                ui.label("Image Extension");
            },
        );

        ui.separator();

        settings_ui_field(
            ui,
            &mut self.max_tile_zoom,
            default.max_tile_zoom,
            Some("The highest zoom level"),
            |ui, value| {
                ui.add(egui::Slider::new(value, 0..=15).text("Maximum tile zoom"));
            },
        );

        settings_ui_field(
            ui,
            &mut self.max_zoom_world_size,
            default.max_zoom_world_size,
            Some(
                "The world size across the width/height of tiles of the maximum zoom\nFor example, a value of 32 and maximum zoom of 8 means tiles of zoom level 8 represent 32 tiles in the world",
            ),
            |ui, value| {
                ui.add(
                    egui::Slider::new(value, 1.0..=256.0)
                        .text("Maximum tile zoom range")
                        .max_decimals(5),
                );
            },
        );

        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.offset != default.offset, egui::Button::new("⟲"))
                .on_hover_text(format!("Default: {:?}", default.offset))
                .clicked()
            {
                self.offset = default.offset;
            }

            ui.label("x +=");
            ui.add(egui::DragValue::new(&mut self.offset.0).suffix("u"));
            ui.label("y +=");
            ui.add(egui::DragValue::new(&mut self.offset.1).suffix("u"));

            ui.label("Tile offset");
        });
        ui.label("Distance in world units to offset tiles by. Positive number means rightwards or downwards");
    }
}
