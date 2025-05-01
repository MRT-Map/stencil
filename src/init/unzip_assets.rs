use std::io::Cursor;

use bevy::prelude::*;
use tracing::info;
use zip::ZipArchive;

use crate::{dirs_paths::data_dir, state::LoadingState};

pub fn unzip_assets_sy(mut commands: Commands) -> Result {
    info!("Unzipping assets to data directory");
    let mut zip_file = ZipArchive::new(Cursor::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/assets.zip"
    ))))?;
    let dir = data_dir("assets");
    zip_file.extract(&dir)?;

    commands.insert_resource(NextState::Pending(LoadingState::UnzipAssets.next()));
    Ok(())
}
