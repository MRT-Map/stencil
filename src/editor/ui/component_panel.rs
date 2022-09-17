use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use itertools::Itertools;

use crate::{
    editor::{
        bundles::component::SelectedComponent,
        ui::HoveringOverGui,
    },
    types::pla::EditorCoords,
};
use crate::types::pla::PlaComponent;
use crate::types::skin::Skin;

pub fn ui(
    mut ctx: ResMut<EguiContext>,
    mut selected: Query<&mut PlaComponent<EditorCoords>, With<SelectedComponent>>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
    skin: Res<Skin>,
) {
    let panel = egui::SidePanel::left("component_data")
        .default_width(200.0)
        .show(ctx.ctx_mut(), |ui| {
            if selected.is_empty() {
                ui.heading("Select a component...");
                return;
            }
            let mut component_data =
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
            let component_type = component_data.get_type(&skin).unwrap();
            let type_selector = egui::ComboBox::from_label("Component type")
                .selected_text(component_data.type_.to_owned())
                .show_ui(ui, |ui| {
                    let responses = skin.types.iter()
                        .filter(|(_, data)| data.get_type() == component_type)
                        .map(|(name, _)| ui.selectable_value(
                            &mut component_data.type_, name.to_owned(), name,
                        ));
                    if responses.collect::<Vec<_>>().into_iter().any(|res| res.hovered()) {
                        hovering_over_gui.0 = true;
                    };
                });
            if type_selector.response.hovered() {
                hovering_over_gui.0 = true;
            };
            ui.end_row();
            let mut tags = component_data.tags.join(",");
            ui.add(egui::TextEdit::singleline(&mut tags).hint_text("Tags"));
            component_data.tags = tags.split(',').map(|t| t.trim().to_string()).collect();
            ui.end_row();
            ui.add(egui::Slider::new(&mut component_data.layer, -10.0..=10.0).text("Layer"));
            ui.end_row();
            ui.separator();
            ui.heading("Position data");
            ui.label(
                component_data.nodes
                    .iter()
                    .map(|a| format!("{}, {}", a.0.x, -a.0.y))
                    .join("\n"),
            );
        });
    if panel.response.hovered() {
        hovering_over_gui.0 = true;
    }
}
