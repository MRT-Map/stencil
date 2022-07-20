use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use crate::HoveringOverGui;

pub fn ui(mut ctx: ResMut<EguiContext>, mut hovering: ResMut<HoveringOverGui>) {
    let mut current_value = "";
    if egui::TopBottomPanel::top("toolbar").show(ctx.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            ui.selectable_value(&mut current_value, "point", "Point");
            ui.selectable_value(&mut current_value, "line", "Line");
            ui.selectable_value(&mut current_value, "area", "Area");
        });
    }).response.hovered() {
        hovering.0 = true
    }
}
