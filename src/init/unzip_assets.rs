use std::io::Cursor;

use bevy::prelude::{Commands, NextState};
use tracing::info;
use zip::ZipArchive;

use crate::{misc::data_dir, state::LoadingState};

#[allow(clippy::needless_pass_by_value)]
pub fn unzip_assets_sy(mut commands: Commands) {
    info!("Unzipping assets to data directory");
    let mut zip_file = ZipArchive::new(Cursor::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/assets.zip"
    ))))
    .unwrap();
    let dir = data_dir("assets");
    zip_file.extract(&dir).unwrap();

    commands.insert_resource(NextState(Some(LoadingState::UnzipAssets.next())));
}
