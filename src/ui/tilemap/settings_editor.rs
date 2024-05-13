use std::time::SystemTime;

use bevy::prelude::*;
use bevy_egui::{egui, egui::Color32};
use egui_file_dialog::FileDialog;
use surf::Url;

use crate::{
    misc::{data_path, Action},
    tile::tile_coord::URL_REPLACER,
    ui::{
        panel::dock::{DockWindow, PanelDockState, PanelParams, TabViewer},
        popup::Popup,
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
        let PanelParams {
            tile_settings,
            status,
            popup,
            ..
        } = &mut tab_viewer.params;
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

        let file_dialog = &mut tab_viewer.file_dialogs.tile_settings_export;
        for (i, basemap) in tile_settings.basemaps.iter_mut().enumerate() {
            ui.separator();
            egui::menu::bar(ui, |ui| {
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
                if ui.button("Export").clicked() {
                    let mut fd = Self::export_dialog(&basemap.url);
                    fd.save_file();
                    *file_dialog = Some((basemap.to_owned(), fd));
                }
            });

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

        if let Some((basemap, file_dialog)) = file_dialog {
            file_dialog.update(ui.ctx());
            if let Some(file) = file_dialog.take_selected() {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                match toml::to_string_pretty(basemap).map(|s| std::fs::write(&file, s)) {
                    Ok(Ok(_)) => {
                        status.0 = format!("Exported basemap to {}", file.to_string_lossy()).into();
                    }
                    Ok(Err(e)) => {
                        popup.send(Popup::base_alert(
                            format!("basemap_write_error_{timestamp}"),
                            "Could not write basemap file",
                            format!(
                                "Could not write basemap file {}:\n{e}",
                                file.to_string_lossy()
                            ),
                        ));
                    }
                    Err(e) => {
                        popup.send(Popup::base_alert(
                            format!("basemap_serialise_error_{timestamp}"),
                            "Could not serialise basemap file",
                            format!(
                                "Could not serialise basemap file {}:\n{e}",
                                file.to_string_lossy()
                            ),
                        ));
                    }
                };
            }
        }

        if new_map != 0 {
            tile_settings.basemaps.swap(0, new_map);
        }
        if let Some(delete) = delete {
            tile_settings.basemaps.remove(delete);
        }

        ui.separator();
        let file_dialog = &mut tab_viewer.file_dialogs.tile_settings_import;
        egui::menu::bar(ui, |ui| {
            if ui.small_button("Add").clicked() {
                tile_settings.basemaps.push(Basemap::default());
            }
            if ui.button("Import").clicked() {
                file_dialog.select_file();
            }
        });
        file_dialog.update(ui.ctx());
        if let Some(file) = file_dialog.take_selected() {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            match std::fs::read_to_string(&file).map(|content| toml::from_str(&content)) {
                Ok(Ok(new)) => {
                    tile_settings.basemaps.insert(0, new);
                    status.0 = format!("Loaded new basemap from {}", file.to_string_lossy()).into();
                }
                Ok(Err(e)) => {
                    popup.send(Popup::base_alert(
                        format!("basemap_parse_error_{timestamp}"),
                        "Could not parse basemap file",
                        format!(
                            "Could not parse basemap file {}:\n{e}",
                            file.to_string_lossy()
                        ),
                    ));
                }
                Err(e) => {
                    popup.send(Popup::base_alert(
                        format!("basemap_read_error_{timestamp}"),
                        "Could not load basemap file",
                        format!(
                            "Could not load basemap file {}:\n{e}",
                            file.to_string_lossy()
                        ),
                    ));
                }
            };
        }

        if !invalid && old_settings != **tile_settings {
            tile_settings.save().unwrap();
        }
    }
}

impl TileSettingsEditor {
    #[must_use]
    pub fn import_dialog() -> FileDialog {
        FileDialog::new().title("Import Basemap")
    }

    #[must_use]
    pub fn export_dialog(url: &str) -> FileDialog {
        FileDialog::new()
            .title(&format!("Export basemap {url}"))
            .default_file_name(&format!(
                "{}.toml",
                URL_REPLACER.replace_all(url, "").as_ref()
            ))
    }
}

pub fn tile_settings_msy(mut actions: EventReader<Action>, mut state: ResMut<PanelDockState>) {
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(OpenTileSettingsAct))
            && !state
                .state
                .iter_all_tabs()
                .any(|(_, a)| a.title() == TileSettingsEditor.title())
        {
            state.state.add_window(vec![TileSettingsEditor.into()]);
        }
    }
}
