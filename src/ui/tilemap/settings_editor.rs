use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_file_dialog::FileDialog;
use egui_notify::ToastLevel;
use surf::Url;

use crate::{
    action::Action,
    dirs_paths::data_path,
    file::{load_toml, save_toml},
    misc_config::settings_editor::{MiscSettingsEditor, OpenMiscSettingsAct},
    notification::{NotifLogRwLockExt, NOTIF_LOG},
    tile::tile_coord::URL_REPLACER,
    ui::{
        panel::dock::{
            window_action_handler, DockWindow, FileDialogs, PanelDockState, PanelParams, TabViewer,
        },
        tilemap::settings::{Basemap, TileSettings},
    },
};

pub enum TileSettingsAct {
    Open,
    Import,
    Export(Basemap),
}

#[derive(Clone, Copy)]
pub struct TileSettingsEditor;

impl DockWindow for TileSettingsEditor {
    fn title(self) -> String {
        "Tile Settings".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams {
            tile_settings,
            actions,
            ..
        } = tab_viewer.params;
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
            egui::Color32::YELLOW,
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
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::YELLOW, format!("#{i}"));
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
                    actions.send(Action::new(TileSettingsAct::Export(basemap.to_owned())));
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
                ui.colored_label(egui::Color32::RED, format!("Invalid URL: {e:?}"));
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
        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                tile_settings.basemaps.push(Basemap::default());
            }
            if ui.button("Import").clicked() {
                actions.send(Action::new(TileSettingsAct::Import));
            }
        });

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

pub fn tile_settings_asy(
    mut actions: EventReader<Action>,
    mut state: ResMut<PanelDockState>,
    mut tile_settings: ResMut<TileSettings>,
    mut ctx: EguiContexts,
    mut file_dialogs: NonSendMut<FileDialogs>,
) {
    let Some(ctx) = ctx.try_ctx_mut() else {
        return;
    };
    for event in actions.read() {
        window_action_handler(
            &event,
            &mut state,
            TileSettingsAct::Open,
            TileSettingsEditor,
        );

        if matches!(event.downcast_ref(), Some(TileSettingsAct::Import)) {
            file_dialogs.tile_settings_import.select_file();
        } else if let Some(TileSettingsAct::Export(basemap)) = event.downcast_ref() {
            let mut fd = TileSettingsEditor::export_dialog(&basemap.url);
            fd.save_file();
            file_dialogs.tile_settings_export = Some((basemap.to_owned(), fd));
        }
    }

    let file_dialog = &mut file_dialogs.tile_settings_import;
    file_dialog.update(ctx);
    if let Some(file) = file_dialog.take_selected() {
        if let Ok(new) = load_toml(&file, Some("basemap")) {
            tile_settings.basemaps.insert(0, new);
            NOTIF_LOG.push(
                &format!("Loaded new basemap from {}", file.to_string_lossy()),
                ToastLevel::Success,
            );
        }
    }

    if let Some((basemap, file_dialog)) = &mut file_dialogs.tile_settings_export {
        file_dialog.update(ctx);
        if let Some(file) = file_dialog.take_selected() {
            if save_toml(basemap, &file, Some("basemap")).is_ok() {
                NOTIF_LOG.push(
                    &format!("Exported basemap to {}", file.to_string_lossy()),
                    ToastLevel::Success,
                );
            }
        }
    }
}
