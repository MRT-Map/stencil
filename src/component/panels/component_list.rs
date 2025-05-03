use bevy::prelude::*;
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    component::{actions::selecting::SelectEv, tools::deleting::DeleteEv},
    ui::panel::dock::{open_dock_window, DockLayout, DockWindow, PanelParams},
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ComponentList;

#[derive(Clone, Copy, Event)]
pub struct OpenComponentListEv;

impl DockWindow for ComponentList {
    fn title(self) -> String {
        "Component List".into()
    }
    fn ui(self, params: &mut PanelParams, ui: &mut egui::Ui) {
        let PanelParams {
            queries,
            camera,
            commands,
            skin,
            ..
        } = params;
        let mut transform = camera.single_mut().unwrap();
        let query = queries.p1();
        let components = query.iter().into_group_map_by(|(_, a)| a.namespace.clone());
        for (ns, components) in components.iter().sorted_by_key(|(a, _)| *a) {
            ui.collapsing(ns, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::auto().at_least(100.0))
                    .column(Column::auto().at_least(50.0))
                    .columns(Column::auto().at_least(10.0), 3)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("id");
                        });
                        header.col(|ui| {
                            ui.label("type");
                        });
                    })
                    .body(|mut body| {
                        for (e, component) in components.iter().sorted_by_key(|(_, a)| &a.id) {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(
                                        egui::RichText::new(component.to_string())
                                            .text_style(egui::TextStyle::Small),
                                    );
                                });
                                row.col(|ui| {
                                    let label =
                                        skin.show_type(&component.ty, ui, &egui::TextStyle::Body);
                                    ui.label(label);
                                });

                                let mut see = false;
                                row.col(|ui| {
                                    if component.nodes.is_empty() {
                                        return;
                                    }
                                    if ui.small_button("See").clicked() {
                                        see = true;
                                    }
                                });
                                row.col(|ui| {
                                    if ui.small_button("Select").clicked() {
                                        commands.entity(*e).trigger(SelectEv::SelectOne);
                                        see = true;
                                    }
                                });
                                row.col(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new("❌")
                                                .small()
                                                .fill(egui::Color32::DARK_RED),
                                        )
                                        .clicked()
                                    {
                                        commands.entity(*e).trigger(DeleteEv);
                                    }
                                });
                                if see {
                                    let centre =
                                        component.nodes.iter().map(|a| a.0.as_vec2()).sum::<Vec2>()
                                            / component.nodes.len() as f32;
                                    transform.translation.x = centre.x;
                                    transform.translation.y = centre.y;
                                }
                            });
                        }
                    });
            });
        }
    }
}

pub fn on_component_list(_trigger: Trigger<OpenComponentListEv>, mut state: ResMut<DockLayout>) {
    open_dock_window(&mut state, ComponentList);
}
