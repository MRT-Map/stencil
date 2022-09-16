use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use itertools::Itertools;

use crate::{
    editor::{
        bundles::component::{EditorComponent, SelectedComponent},
        ui::HoveringOverGui,
    },
    types::pla::ComponentCoords,
};

pub fn ui(
    mut ctx: ResMut<EguiContext>,
    mut selected: Query<(&mut EditorComponent, &ComponentCoords), With<SelectedComponent>>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
) {
    let panel = egui::SidePanel::left("component_data")
        .default_width(200.0)
        .show(ctx.ctx_mut(), |ui| {
            if selected.is_empty() {
                ui.heading("Select a component...");
                return;
            }
            let (mut component_data, component_coords): (Mut<EditorComponent>, &ComponentCoords) =
                selected.single_mut();
            ui.heading("Edit component data");
            ui.end_row();
            ui.add(
                egui::TextEdit::singleline(&mut component_data.namespace)
                    .hint_text("ns.")
                    .desired_width(25.0),
            );
            ui.add(egui::TextEdit::singleline(&mut component_data.id).hint_text("id"));
            ui.end_row();
            ui.add(
                egui::TextEdit::singleline(&mut component_data.display_name)
                    .hint_text("Displayed as"),
            );
            ui.end_row();
            ui.add(
                egui::TextEdit::multiline(&mut component_data.description).hint_text("Description"),
            );
            ui.end_row();
            ui.separator();
            egui::ComboBox::from_label("Component type")
                .selected_text(&component_data.type_)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut component_data.type_, "Test".into(), "Test")
                });
            ui.end_row();
            ui.add(egui::TextEdit::singleline(&mut component_data.tags).hint_text("Tags"));
            ui.end_row();
            ui.add(egui::Slider::new(&mut component_data.layer, -10.0..=10.0).text("Layer"));
            ui.end_row();
            ui.separator();
            ui.heading("Position data");
            ui.label(
                component_coords
                    .0
                    .iter()
                    .map(|a| format!("{}, {}", a.x, -a.y))
                    .join("\n"),
            );
        });
    if panel.response.hovered() {
        hovering_over_gui.0 = true;
    }
}
