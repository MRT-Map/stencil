use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::{menu, menu_actions::MenuAction, ui::popup::Popup};

pub fn changelog_msy(mut events: EventReader<MenuAction>, mut popup: EventWriter<Arc<Popup>>) {
    menu!(events, "changelog");
    popup.send(Arc::new(Popup {
        id: "changelog",
        window: Box::new(|| {
            egui::Window::new("Changelog")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        }),
        ui: Box::new(|ui, _, show| {
            let mut cache = CommonMarkCache::default();
            CommonMarkViewer::new("viewer").show(
                ui,
                &mut cache,
                include_str!("../../changelog.md"),
            );
            if ui.button("Close").clicked() {
                *show = false;
            }
        }),
    }));
}
