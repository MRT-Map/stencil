use bevy::{prelude::*, render::settings::Backends, window::WindowMode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use surf::Url;
use tracing::info;

use crate::{
    dirs_paths::data_path,
    load_save::{load_toml, save_toml_with_header},
};

macro_rules! field {
    ($s:ty, $f:ident, $f2:ident, $i:ident, $t:ty) => {
        #[allow(clippy::float_cmp)]
        fn $f(v: &$t) -> bool {
            *v == <$s>::default().$i
        }
        fn $f2() -> $t {
            <$s>::default().$i
        }
    };
}
field!(
    MiscSettings,
    skin_url_is_default,
    default_skin_url,
    skin_url,
    String
);

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Resource)]
pub struct MiscSettings {
    pub skin_url: String,
}

impl Default for MiscSettings {
    fn default() -> Self {
        Self {
            skin_url: "https://raw.githubusercontent.com/MRT-Map/tile-renderer/main/renderer/skins/default.json".into()
        }
    }
}

impl MiscSettings {
    pub fn load() -> Self {
        if !data_path("misc_settings.toml").exists() {
            let s = Self::default();
            let _ = s.save();
            return s;
        }
        match load_toml(&data_path("misc_settings.toml"), Some("misc settings")) {
            Ok(str) => {
                info!("Found misc settings file");
                str
            }
            Err(e) => {
                info!("Couldn't open or parse misc settings file: {e:?}");

                Self::default()
            }
        }
    }
    pub fn save(&self) -> eyre::Result<()> {
        save_toml_with_header(self, &data_path("misc_settings.toml"), "# Documentation is at https://github.com/MRT-Map/stencil2/wiki/Advanced-Topics#misc_settingstoml", Some("misc settings"))
    }
}

pub static INIT_MISC_SETTINGS: Lazy<MiscSettings> = Lazy::new(MiscSettings::load);
