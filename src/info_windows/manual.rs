use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::{info_windows::InfoWindowsAct, misc::Action, ui::popup::Popup};

pub fn manual_asy(mut actions: EventReader<Action>, mut popup: EventWriter<Arc<Popup>>) {
    for event in actions.iter() {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Manual)) {
            popup.send(Popup::new(
                "manual",
                || {
                    egui::Window::new("Manual")
                        .collapsible(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                        .vscroll(true)
                },
                |_, ui, _, show| {
                    let mut cache = CommonMarkCache::default();
                    CommonMarkViewer::new("viewer").show(
                        ui,
                        &mut cache,
                        include_str!("../../manual.md"),
                    );
                    if ui.button("Close").clicked() {
                        *show = false;
                    }
                },
                Mutex::new(Box::new(())),
            ));
        }
    }
}
