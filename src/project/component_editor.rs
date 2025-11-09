use std::sync::Arc;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    App,
    project::{pla3::PlaNode, skin::SkinType},
    ui::dock::DockWindow,
};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ComponentEditorWindow;

impl DockWindow for ComponentEditorWindow {
    fn title(&self) -> String {
        "Component Editor".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        let Some(skin) = app.project.skin() else {
            ui.heading("Waiting for skin...");
            return;
        };
        let selected_components_ids = &app.ui.dock_layout.map_window_mut().selected_components;
        let mut selected_components = app
            .project
            .components
            .iter_mut()
            .filter(|a| selected_components_ids.contains(&a.full_id))
            .collect::<Vec<_>>();
        if selected_components.is_empty() {
            ui.heading("Select components...");
            return;
        }

        ui.heading("Edit component data");
        ui.end_row();

        ui.horizontal(|ui| {
            let namespace = selected_components
                .iter()
                .map(|c| &c.full_id.namespace)
                .sorted()
                .dedup()
                .exactly_one()
                .cloned()
                .ok();
            egui::ComboBox::from_label("ns.")
                .selected_text(
                    namespace
                        .as_ref()
                        .map_or_else(|| egui::RichText::new("mixed").italics(), Into::into),
                )
                .width(25.0)
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    for (ns, vis) in &app.project.namespaces {
                        if !vis {
                            continue;
                        }
                        if ui
                            .selectable_label(namespace.as_ref() == Some(ns), ns)
                            .clicked()
                        {
                            for component in &mut selected_components {
                                ns.clone_into(&mut component.full_id.namespace);
                            }
                        }
                    }
                });

            if let Ok(component) = selected_components.iter_mut().exactly_one() {
                ui.add(
                    egui::TextEdit::singleline(&mut component.full_id.id)
                        .hint_text("id")
                        .desired_width(f32::INFINITY),
                );
            } else {
                ui.label(egui::RichText::new("mixed ids").italics());
            }
        });
        ui.end_row();

        let display_name = selected_components
            .iter()
            .map(|c| &c.display_name)
            .sorted()
            .dedup()
            .exactly_one()
            .ok();
        let mut new_display_name = display_name.cloned().unwrap_or_default();
        if ui
            .add(
                egui::TextEdit::singleline(&mut new_display_name)
                    .hint_text(if display_name.is_none() {
                        egui::RichText::new("mixed display names").italics()
                    } else {
                        "Display Name".into()
                    })
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            for component in &mut selected_components {
                new_display_name.clone_into(&mut component.display_name);
            }
        }
        ui.end_row();

        ui.separator();

        let skin_ty = selected_components
            .iter()
            .map(|c| &c.ty)
            .sorted_by_key(|a| a.name())
            .dedup()
            .exactly_one()
            .map(Arc::clone)
            .ok();
        let component_ty = selected_components
            .iter()
            .map(|c| &c.ty)
            .map(|a| match &**a {
                SkinType::Point { .. } => "point",
                SkinType::Line { .. } => "line",
                SkinType::Area { .. } => "area",
            })
            .dedup()
            .exactly_one()
            .ok();
        egui::ComboBox::from_label("Component type")
            .selected_text(skin_ty.as_ref().map_or_else(
                || {
                    egui::RichText::new(if component_ty.is_some() {
                        "mixed skin types"
                    } else {
                        "mixed component types"
                    })
                    .italics()
                    .into()
                },
                |skin_ty| skin_ty.widget_text(ui, &egui::TextStyle::Button).into(),
            ))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                let Some(component_ty) = component_ty else {
                    return;
                };

                skin.types
                    .iter()
                    .filter(|ty| match component_ty {
                        "point" => matches!(&***ty, SkinType::Point { .. }),
                        "line" => matches!(&***ty, SkinType::Line { .. }),
                        "area" => matches!(&***ty, SkinType::Area { .. }),
                        _ => unreachable!(),
                    })
                    .sorted_by_key(|ty| ty.name())
                    .for_each(|ty| {
                        if ui
                            .selectable_label(
                                skin_ty.as_ref().is_some_and(|a| Arc::ptr_eq(a, ty)),
                                ty.widget_text(ui, &egui::TextStyle::Button),
                            )
                            .clicked()
                        {
                            for component in &mut selected_components {
                                component.ty = Arc::clone(ty);
                            }
                        }
                    });
            });
        ui.end_row();

        let layer = selected_components
            .iter()
            .map(|c| c.layer)
            .sorted_by(f32::total_cmp)
            .dedup()
            .exactly_one()
            .ok();
        let mut new_layer = layer.unwrap_or_default();
        if ui
            .add(
                egui::Slider::new(&mut new_layer, -10.0..=10.0).text(if layer.is_none() {
                    egui::RichText::new("Mixed Layers").italics()
                } else {
                    "Layer".into()
                }),
            )
            .changed()
        {
            for component in &mut selected_components {
                component.layer = new_layer;
            }
        }

        ui.end_row();
        ui.separator();

        if component_ty == Some("line") {
            if ui.button("Reverse direction").clicked() {
                for _component in &mut selected_components {
                    // TODO reverse
                }
            }
            ui.end_row();
            ui.separator();
        }

        let Ok(component) = selected_components.iter().exactly_one() else {
            return;
        };
        ui.heading("Position data");
        let is_line = matches!(&*component.ty, SkinType::Line { .. });
        egui_extras::TableBuilder::new(ui)
            .columns(egui_extras::Column::auto().at_least(50.0), 4)
            .cell_layout(egui::Layout::default().with_cross_align(egui::Align::RIGHT))
            .header(10.0, |mut header| {
                header.col(|_| ());
                header.col(|ui| {
                    ui.label("X");
                });
                header.col(|ui| {
                    ui.label("Y");
                });
            })
            .body(|mut body| {
                let mut add_row =
                    |ty: &str, coord: geo::Coord<i32>, colour: egui::Color32, label: Option<u8>| {
                        body.row(10.0, |mut row| {
                            row.col(|ui| {
                                ui.label(ty);
                            });
                            row.col(|ui| {
                                ui.label(
                                    egui::RichText::new(coord.x.to_string())
                                        .color(colour)
                                        .monospace(),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    egui::RichText::new(coord.y.to_string())
                                        .color(colour)
                                        .monospace(),
                                );
                            });
                            if let Some(label) = label {
                                row.col(|ui| {
                                    ui.label(format!("@ {label}"));
                                });
                            }
                        });
                    };
                for (i, node) in component.nodes.iter().enumerate() {
                    let colour = if i == 0 && is_line {
                        egui::Color32::GREEN
                    } else if i == component.nodes.len() - 1 && is_line {
                        egui::Color32::RED
                    } else {
                        egui::Color32::WHITE
                    };
                    match *node {
                        PlaNode::Line { coord, label } => {
                            add_row("line", coord, colour, label);
                        }
                        PlaNode::QuadraticBezier { ctrl, coord, label } => {
                            add_row("ctrl", ctrl, egui::Color32::WHITE, None);
                            add_row("quad", coord, colour, label);
                        }
                        PlaNode::CubicBezier {
                            ctrl1,
                            ctrl2,
                            coord,
                            label,
                        } => {
                            add_row("ctrl1", ctrl1, egui::Color32::WHITE, None);
                            add_row("ctrl2", ctrl2, egui::Color32::WHITE, None);
                            add_row("cubic", coord, colour, label);
                        }
                    }
                }
            });
    }
}
