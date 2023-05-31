use bevy::prelude::*;

use crate::{misc::data_file, state::LoadingState, tile::settings::TileSettings};

fn v2_0_1() {
    info!("Running compatibility upgrades from v2.0.1");
    if data_file("tile_settings.msgpack").is_dir() {
        let _ = std::fs::remove_dir_all(data_file("tile_settings.msgpack"));
    }
}

fn v2_1_0() {
    info!("Running compatibility upgrades from v2.1.0");
    if let Ok(b) = std::fs::read(data_file("tile_settings.msgpack")) {
        if let Ok(t) = rmp_serde::from_slice::<TileSettings>(&b) {
            let _ = t.save();
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn compat_sy(mut commands: Commands) {
    v2_0_1();
    v2_1_0();

    commands.insert_resource(NextState(Some(LoadingState::Compat.next())));
}
