use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::{egui, egui::vec2, EguiContexts};

use crate::{
    info_windows::InfoWindowsAct, init::load_assets::ImageAssets, misc::Action, ui::popup::Popup,
};

#[allow(clippy::needless_pass_by_value)]
pub fn info_asy(
    mut actions: EventReader<Action>,
    mut popup: EventWriter<Popup>,
    _images: Res<ImageAssets>,
    mut ctx: EguiContexts,
) {
    egui_extras::install_image_loaders(ctx.ctx_mut());
    for event in actions.read() {
        if matches!(event.downcast_ref(), Some(InfoWindowsAct::Info)) {
            popup.send(Popup::new(
                "info",
                || {
                    egui::Window::new(format!("Stencil v{}", env!("CARGO_PKG_VERSION")))
                        .collapsible(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                },
                move |_, ui, _, shown| {
                    ui.add(
                        egui::Image::new(egui::include_image!("../../assets/ste-light.png"))
                            .fit_to_exact_size(vec2(975.0 / 4.0, 569.0 / 4.0)),
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
    }
}
