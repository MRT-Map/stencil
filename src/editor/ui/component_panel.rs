use bevy::prelude::*;
use bevy_egui::{egui, egui::Pos2, EguiContext};
use bevy_mouse_tracking_plugin::MousePos;
use itertools::Itertools;

use crate::{
    editor::{bundles::component::SelectedComponent, ui::HoveringOverGui},
    types::{
        pla::{EditorCoords, PlaComponent},
        skin::Skin,
        ComponentType,
    },
};

#[derive(Default)]
pub struct PrevNamespaceUsed(pub String);

pub fn ui_sy(
    mut ctx: ResMut<EguiContext>,
    mut selected: Query<(Entity, &mut PlaComponent<EditorCoords>), With<SelectedComponent>>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
    mut commands: Commands,
    skin: Res<Skin>,
    mouse_pos: Res<MousePos>,
    mut prev_namespace_used: ResMut<PrevNamespaceUsed>,
) {
    let panel = egui::SidePanel::left("component_data")
        .default_width(200.0)
        .show(ctx.ctx_mut(), |ui| {
            if selected.is_empty() {
                ui.heading("Select a component...");
                return;
            }
            let (entity, mut component_data): (Entity, Mut<PlaComponent<EditorCoords>>) =
                selected.single_mut();
            ui.heading("Edit component data");
            ui.end_row();
            ui.add(
                egui::TextEdit::singleline(&mut component_data.namespace)
                    .hint_text("ns.")
                    .desired_width(25.0),
            );
            prev_namespace_used.0 = component_data.namespace.to_owned();
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
            let old_skin_type = component_data.ty.to_owned();
            egui::ComboBox::from_label("Component type")
                .selected_text(component_data.ty.to_owned())
                .show_ui(ui, |ui| {
                    skin.types
                        .iter()
                        .filter(|(_, data)| data.get_type() == component_type)
                        .map(|(name, _)| {
                            ui.selectable_value(&mut component_data.ty, name.to_owned(), name)
                        })
                        .for_each(|_| ());
                });
            if old_skin_type != component_data.ty {
                commands
                    .entity(entity)
                    .insert_bundle(component_data.get_shape(&skin, true));
            }
            ui.end_row();
            let mut tags = component_data.tags.join(",");
            ui.add(egui::TextEdit::singleline(&mut tags).hint_text("Tags"));
            component_data.tags = tags.split(',').map(|t| t.trim().to_string()).collect();
            ui.end_row();
            ui.add(egui::Slider::new(&mut component_data.layer, -10.0..=10.0).text("Layer"));
            ui.end_row();
            ui.separator();
            if component_data.get_type(&skin) == Some(ComponentType::Line) {
                if ui.button("Reverse direction").clicked() {
                    component_data.nodes.reverse();
                };
                ui.end_row();
                ui.separator();
            }
            ui.heading("Position data");
            ui.label(
                component_data
                    .nodes
                    .iter()
                    .map(|a| format!("{}, {}", a.0.x, -a.0.y))
                    .join("\n"),
            );
        });
    if panel
        .response
        .rect
        .contains(Pos2::new(mouse_pos.x, mouse_pos.y))
    {
        hovering_over_gui.0 = true;
    }
}
