use bevy::prelude::*;
use bevy_egui::egui;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    component::{actions::rendering::RenderEv, pla2::ComponentType},
    history::{HistoryEntry, HistoryEv},
    ui::panel::dock::{open_dock_window, DockLayout, DockWindow, PanelParams},
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ComponentEditor;

#[derive(Clone, Copy, Event)]
pub struct OpenComponentEditorEv;

impl DockWindow for ComponentEditor {
    fn title(self) -> String {
        "Component".into()
    }
    fn ui(self, params: &mut PanelParams, ui: &mut egui::Ui) {
        let PanelParams {
            queries,
            commands,
            skin,
            namespaces,
            ..
        } = params;
        let mut selected = queries.p0();
        if selected.is_empty() {
            ui.heading("Select a component...");
            return;
        }
        let (e, mut component_data) = selected.single_mut().unwrap();
        let old_data = component_data.to_owned();
        ui.heading("Edit component data");
        ui.end_row();
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("ns.")
                .selected_text(&component_data.namespace)
                .width(25.0)
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    for (ns, vis) in &namespaces.visibilities {
                        if !vis {
                            continue;
                        }
                        ui.selectable_value(&mut component_data.namespace, ns.to_owned(), ns);
                    }
                });
            component_data
                .namespace
                .clone_into(&mut namespaces.prev_used);
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
        let component_type = component_data.get_type(skin);
        let old_skin_type = component_data.ty.clone();
        egui::ComboBox::from_label("Component type")
            .selected_text(component_data.ty.clone())
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                skin.types
                    .iter()
                    .filter(|skin_comp| skin_comp.get_type() == component_type)
                    .sorted_by_key(|skin_comp| skin_comp.name())
                    .for_each(|skin_comp| {
                        ui.selectable_value(
                            &mut component_data.ty,
                            skin_comp.name().to_owned(),
                            skin_comp.name(),
                        );
                    });
            });
        if old_skin_type != component_data.ty {
            commands.entity(e).trigger(RenderEv::default());
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
        if component_data.get_type(skin) == ComponentType::Line {
            if ui.button("Reverse direction").clicked() {
                component_data.nodes.reverse();
            }
            ui.end_row();
            ui.separator();
        }
        ui.heading("Position data");
        let is_line = component_data.get_type(skin) == ComponentType::Line;
        for (i, a) in component_data.nodes.iter().enumerate() {
            let color = if i == 0 && is_line {
                egui::Color32::GREEN
            } else if i == component_data.nodes.len() - 1 && is_line {
                egui::Color32::RED
            } else {
                egui::Color32::WHITE
            };
            ui.colored_label(color, format!("{}, {}", a.0.x, -a.0.y));
        }
        if *component_data != old_data {
            commands.trigger(HistoryEv::one_history(HistoryEntry::Component {
                e,
                before: Some(old_data.into()),
                after: Some(component_data.to_owned().into()),
            }));
        }
    }
}

pub fn on_component_editor(
    _trigger: Trigger<OpenComponentEditorEv>,
    mut state: ResMut<DockLayout>,
) {
    open_dock_window(&mut state, ComponentEditor);
}
