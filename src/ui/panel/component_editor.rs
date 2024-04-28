use bevy::prelude::*;
use bevy_egui::egui;
use itertools::Itertools;

use crate::{
    component_actions::undo_redo::{History, UndoRedoAct},
    misc::Action,
    pla2::{bundle::EntityCommandsSelectExt, component::ComponentType},
    ui::panel::dock::{DockWindow, PanelParams, TabViewer},
};

#[derive(Default, Resource, Clone)]
pub struct PrevNamespaceUsed(pub String);

#[derive(Clone, Copy)]
pub struct ComponentEditor;

impl DockWindow for ComponentEditor {
    fn title(self) -> String {
        "Component".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams {
            selected,
            commands,
            skin,
            prev_namespace_used,
            actions,
            ..
        } = &mut tab_viewer.params;
        if selected.is_empty() {
            ui.heading("Select a component...");
            return;
        }
        let (entity, mut component_data) = selected.single_mut();
        let old_data = component_data.to_owned();
        ui.heading("Edit component data");
        ui.end_row();
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut component_data.namespace)
                    .hint_text("ns.")
                    .desired_width(25.0),
            );
            component_data
                .namespace
                .clone_into(&mut prev_namespace_used.0);
            ui.add(egui::TextEdit::singleline(&mut component_data.id).hint_text("id"));
        });
        ui.end_row();
        ui.add(
            egui::TextEdit::singleline(&mut component_data.display_name).hint_text("Displayed as"),
        );
        ui.end_row();
        ui.add(egui::TextEdit::multiline(&mut component_data.description).hint_text("Description"));
        ui.end_row();
        ui.separator();
        let component_type = component_data.get_type(skin).unwrap();
        let old_skin_type = component_data.ty.to_owned();
        egui::ComboBox::from_label("Component type")
            .selected_text(component_data.ty.to_owned())
            .show_ui(ui, |ui| {
                skin.types
                    .iter()
                    .filter(|(_, data)| data.get_type() == component_type)
                    .sorted_by_key(|(name, _)| *name)
                    .for_each(|(name, _)| {
                        ui.selectable_value(&mut component_data.ty, name.to_owned(), name);
                    });
            });
        if old_skin_type != component_data.ty {
            commands
                .entity(entity)
                .select_component(skin, &component_data);
        }
        ui.end_row();
        let mut tags = component_data.tags.join(",");
        ui.add(
            egui::TextEdit::singleline(&mut tags)
                .hint_text("Tags")
                .desired_width(f32::INFINITY),
        );
        component_data.tags = tags.split(',').map(|t| t.trim().to_owned()).collect();
        ui.end_row();
        ui.add(egui::Slider::new(&mut component_data.layer, -10.0..=10.0).text("Layer"));
        ui.end_row();
        ui.separator();
        if component_data.get_type(skin) == Some(ComponentType::Line) {
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
        if *component_data != old_data {
            actions.send(Action::new(UndoRedoAct::one_history(History {
                component_id: entity,
                before: Some(old_data),
                after: Some(component_data.to_owned()),
            })));
        }
    }
    fn closeable(self) -> bool {
        false
    }
}
