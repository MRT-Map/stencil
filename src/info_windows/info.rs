use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::{egui, egui::TextureId, EguiContexts};

use crate::{info_windows::InfoWindowsAct, misc::Action, ui::popup::Popup};

#[allow(clippy::needless_pass_by_value)]
pub fn info_asy(
    mut actions: EventReader<Action>,
    mut popup: EventWriter<Arc<Popup>>,
    server: Res<AssetServer>,
    mut ctx: EguiContexts,
    mut texture: Local<Option<TextureId>>,
) {
    let texture = texture
        .get_or_insert_with(|| ctx.add_image(server.load("stencil-text.png")))
        .to_owned();
    for event in actions.iter() {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Info)) {
            popup.send(Popup::new(
                "info_popup",
                || {
                    egui::Window::new(format!("Stencil v{}", env!("CARGO_PKG_VERSION")))
                        .collapsible(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                },
                move |_, ui, _, shown| {
                    ui.image(texture, [975.0 / 4.0, 569.0 / 4.0]);
                    ui.label("Made by __7d for the MRT Mapping Services");
                    ui.hyperlink_to("GitHub", "https://github.com/MRT-Map/stencil2");
                    if ui.button("Close").clicked() {
                        *shown = false;
                    }
                },
                Mutex::new(Box::new(())),
            ));
        }
    }
}
