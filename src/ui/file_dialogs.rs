use bevy::prelude::*;
use egui_file_dialog::{FileDialog, FileDialogStorage};
use tracing::info;

use crate::{
    dirs_paths::cache_path,
    file::{load_toml, save_toml},
    project::project_editor::ProjectEditor,
    ui::tilemap::{settings::Basemap, settings_editor::TileSettingsEditor},
};

#[derive(Debug, Resource)]
pub struct FileDialogs {
    pub tile_settings_import: FileDialog,
    pub tile_settings_export: Option<(Basemap, FileDialog)>,
    pub project_select: FileDialog,
}

impl Default for FileDialogs {
    fn default() -> Self {
        Self {
            tile_settings_import: TileSettingsEditor::import_dialog(),
            tile_settings_export: None,
            project_select: ProjectEditor::select_dialog(),
        }
    }
}

impl FileDialogs {
    pub fn load_storage() -> FileDialogStorage {
        if !cache_path("file_dialog_storage.toml").exists() {
            let s = FileDialogStorage::default();
            let _ = Self::save_storage(&s);
            return s;
        }
        match load_toml(
            &cache_path("file_dialog_storage.toml"),
            Some("file dialog storage"),
        ) {
            Ok(str) => {
                info!("Found file dialog storage file");
                str
            }
            Err(e) => {
                info!("Couldn't open or parse file dialog storage file: {e:?}");

                FileDialogStorage::default()
            }
        }
    }
    pub fn save_storage(storage: &FileDialogStorage) -> eyre::Result<()> {
        save_toml(
            storage,
            &cache_path("file_dialog_storage.toml"),
            Some("file dialog storage"),
        )
    }
}
