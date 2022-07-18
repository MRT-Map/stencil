use crate::ResMut;
use bevy_egui::{egui, EguiContext};
use std::collections::HashMap;

#[derive(Default)]
pub struct CurrentComponentData {
    namespace: String,
    id: String,
    display_name: String,
    description: String,
    tags: String,
    layer: f64,
    type_: String,
    attributes: HashMap<String, String>,
}

pub fn ui(mut ctx: ResMut<EguiContext>, mut current_comp_data: ResMut<CurrentComponentData>) {
    egui::SidePanel::left("main")
        .default_width(200.0)
        .show(ctx.ctx_mut(), |ui| {
            ui.heading("Edit component data");
            ui.end_row();
            ui.add(
                egui::TextEdit::singleline(&mut current_comp_data.namespace)
                    .hint_text("ns.")
                    .desired_width(25.0),
            );
            ui.add(egui::TextEdit::singleline(&mut current_comp_data.id).hint_text("id"));
            ui.end_row();
            ui.add(
                egui::TextEdit::singleline(&mut current_comp_data.display_name)
                    .hint_text("Displayed as"),
            );
            ui.end_row();
            ui.add(
                egui::TextEdit::multiline(&mut current_comp_data.description)
                    .hint_text("Description"),
            );
            ui.end_row();
            ui.separator();
            egui::ComboBox::from_label("Component type")
                .selected_text(&current_comp_data.type_)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut current_comp_data.type_, "Test".into(), "Test")
                });
            ui.end_row();
            ui.add(egui::TextEdit::singleline(&mut current_comp_data.tags).hint_text("Tags"));
            ui.end_row();
            ui.add(egui::Slider::new(&mut current_comp_data.layer, -10.0..=10.0).text("Layer"));
        });
}
