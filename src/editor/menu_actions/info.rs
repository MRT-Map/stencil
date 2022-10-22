use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{editor::menu_actions::MenuAction, menu};

pub fn info_msy(
    mut events: EventReader<MenuAction>,
    mut ctx: ResMut<EguiContext>,
    mut show_window: Local<bool>,
) {
    if *show_window {
        egui::Window::new(&format!("Stencil v{}", env!("CARGO_PKG_VERSION")))
            .collapsible(false)
            .show(ctx.ctx_mut(), |ui| {
                ui.label("Made by __7d for the MRT Mapping Services");
                ui.label("Changelogs would appear here...");
                if ui.button("Close").clicked() {
                    *show_window = false
                }
            });
    }
    menu!(events, "info");
    *show_window = true;
}
