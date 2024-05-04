use bevy::prelude::*;
use bevy_egui::{egui, egui::Color32};
use surf::Url;

use crate::{
    misc::{data_path, Action},
    ui::{
        panel::dock::{DockWindow, PanelDockState, PanelParams, TabViewer},
        tilemap::settings::{Basemap, TileSettings},
    },
};

#[allow(dead_code)]
pub struct OpenTileSettingsAct;

#[derive(Clone, Copy)]
pub struct TileSettingsEditor;

impl DockWindow for TileSettingsEditor {
    fn title(self) -> String {
        "Tile Settings".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams { tile_settings, .. } = &mut tab_viewer.params;
        let mut invalid = false;
        let old_settings = tile_settings.to_owned();

        if ui
            .add_enabled(
                **tile_settings != TileSettings::default(),
                egui::Button::new("Reset"),
            )
            .clicked()
        {
            **tile_settings = TileSettings::default();
        }
        ui.colored_label(
            Color32::YELLOW,
            format!(
                "Tile settings can also be edited at: {}",
                data_path("tile_settings.toml").to_string_lossy()
            ),
        );
        ui.separator();

        ui.add(egui::Slider::new(&mut tile_settings.init_zoom, -10.0..=10.0).text("Initial zoom"));
        ui.label("How zoomed in the map is when the app is first opened. Larger values mean more zoomed in");
        ui.checkbox(
            &mut tile_settings.clear_cache_on_startup,
            "Clear tile cache on startup",
        );
        ui.separator();

        ui.add(
            egui::Slider::new(&mut tile_settings.max_get_requests, 1..=1000)
                .text("Maximum HTTP GET requests"),
        );
        ui.label("Maximum number of tiles to download at a time");
        ui.separator();

        ui.heading("Basemaps");
        ui.label("The top-most entry in this list will be used to render the map");
        let mut new_map = 0;
        let len = tile_settings.basemaps.len();
        let mut delete = None;

        for (i, basemap) in tile_settings.basemaps.iter_mut().enumerate() {
            ui.separator();
            ui.colored_label(Color32::YELLOW, format!("#{i}"));
            if ui
                .add_enabled(i != 0, egui::Button::new("Select"))
                .clicked()
            {
                new_map = i;
            }
            if ui
                .add_enabled(len != 1, egui::Button::new("Delete"))
                .clicked()
            {
                delete = Some(i);
            }

            ui.add(
                egui::Slider::new(&mut basemap.max_tile_zoom, -5..=15).text("Maximum tile zoom"),
            );
            ui.label("...I don't know how to explain this");
            ui.add(
                egui::Slider::new(&mut basemap.max_zoom_range, 1.0..=256.0)
                    .text("Maximum tile zoom range"),
            );
            ui.label("In tiles of the highest zoom level, the distance across its width / height that each tile represents");
            ui.add(egui::TextEdit::singleline(&mut basemap.url).hint_text("Base URL"));
            if let Err(e) = Url::try_from(&*basemap.url) {
                ui.colored_label(Color32::RED, format!("Invalid URL: {e:?}"));
                invalid = true;
            }
            ui.label("The base URL of the tile source");
        }

        if new_map != 0 {
            tile_settings.basemaps.swap(0, new_map);
        }
        if let Some(delete) = delete {
            tile_settings.basemaps.remove(delete);
        }

        ui.separator();
        if ui.button("Add").clicked() {
            tile_settings.basemaps.push(Basemap::default());
        }

        if !invalid && old_settings != **tile_settings {
            tile_settings.save().unwrap();
        }
    }
}

pub fn tile_settings_msy(mut actions: EventReader<Action>, mut state: ResMut<PanelDockState>) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(OpenTileSettingsAct)) {
            state.state.add_window(vec![TileSettingsEditor.into()]);
        }
    }
}
