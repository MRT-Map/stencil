use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    info_windows::InfoWindowsEv,
    ui::popup::{Popup, Popups},
};

#[expect(clippy::needless_pass_by_value)]
pub fn on_info(trigger: Trigger<InfoWindowsEv>, mut popups: ResMut<Popups>) {
    if *trigger.event() != InfoWindowsEv::Info {
        return;
    }
    popups.add(Popup::new(
        "info",
        || {
            egui::Window::new(format!("Stencil v{}", env!("CARGO_PKG_VERSION")))
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        },
        move |_, ui, _, shown| {
            ui.add(
                egui::Image::new(egui::include_image!("../../assets/ste-light.png"))
                    .fit_to_exact_size(egui::vec2(975.0 / 4.0, 569.0 / 4.0)),
            );
            ui.label("Made by __7d for the MRT Mapping Services");
            ui.hyperlink_to("GitHub", "https://github.com/MRT-Map/stencil2");
            if ui.button("Close").clicked() {
                *shown = false;
            }
        },
        Mutex::new(Box::new(())),
    ));
}
