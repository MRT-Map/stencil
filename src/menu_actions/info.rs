use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{menu, menu_actions::MenuAction, ui::popup::Popup};

pub fn info_msy(mut events: EventReader<MenuAction>, mut popup: EventWriter<Arc<Popup>>) {
    menu!(events, "info");
    popup.send(Arc::new(Popup {
        id: "info",
        window: Box::new(|| {
            egui::Window::new(&format!("Stencil v{}", env!("CARGO_PKG_VERSION")))
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        }),
        ui: Box::new(|ui, _, show| {
            ui.label("Made by __7d for the MRT Mapping Services");
            ui.label("Links would appear here...");
            if ui.button("Close").clicked() {
                *show = false;
            }
        }),
    }));
}
