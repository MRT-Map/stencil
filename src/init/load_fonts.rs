use std::{collections::HashMap, sync::Arc};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_notify::ToastLevel;

use crate::{
    dirs_paths::data_dir,
    state::LoadingState,
    ui::notif::{NotifLogRwLockExt, NOTIF_LOG},
};

pub fn get_fonts_sy(mut commands: Commands, mut ctx: EguiContexts) {
    info!("Loading fonts");
    let mut fonts = HashMap::new();
    for result in data_dir("fonts").read_dir().unwrap() {
        let Ok(result) = result else { continue };
        if result.path().extension() != Some("ttf".as_ref())
            && result.path().extension() != Some("otf".as_ref())
        {
            NOTIF_LOG.push(
                &format!("{} is not font file", result.path().to_string_lossy()),
                ToastLevel::Warning,
            );
            continue;
        }
        match std::fs::read(result.path()) {
            Ok(bytes) => {
                fonts.insert(
                    result
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    bytes,
                );
            }
            Err(e) => {
                NOTIF_LOG.push(
                    &format!(
                        "Could not load font file {}:\n{e}",
                        result.path().to_string_lossy()
                    ),
                    ToastLevel::Warning,
                );
            }
        }
    }

    let mut font_definitions = egui::FontDefinitions::default();
    for (name, bytes) in fonts {
        font_definitions
            .font_data
            .insert(name.clone(), Arc::new(egui::FontData::from_owned(bytes)));
        font_definitions
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push(name);
    }
    ctx.ctx_mut().set_fonts(font_definitions);

    commands.insert_resource(NextState::Pending(LoadingState::LoadFonts.next()));
}
