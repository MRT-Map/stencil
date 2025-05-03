use bevy::prelude::*;
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    component::actions::selecting::SelectEv,
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
            ..
        } = params;
        let mut transform = camera.single_mut().unwrap();
        let query = queries.p1();
        let components = query.iter().into_group_map_by(|(_, a)| a.namespace.clone());
        for (ns, components) in components.iter().sorted_by_key(|(a, _)| *a) {
            ui.collapsing(ns, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::auto().at_least(150.0))
                    .column(Column::auto().at_least(50.0))
                    .column(Column::auto().at_least(10.0))
                    .column(Column::auto().at_least(10.0))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("id");
                        });
                        header.col(|ui| {
                            ui.label("type");
                        });
                        header.col(|_| ());
                    })
                    .body(|mut body| {
                        for (e, component) in components {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(component.to_string());
                                });
                                row.col(|ui| {
                                    ui.label(&component.ty);
                                });

                                let mut move_to = false;
                                row.col(|ui| {
                                    if component.nodes.is_empty() {
                                        return;
                                    }
                                    if ui.small_button("Move to").clicked() {
                                        move_to = true;
                                    }
                                });
                                row.col(|ui| {
                                    if ui.small_button("Select").clicked() {
                                        commands.entity(*e).trigger(SelectEv::SelectOne);
                                        move_to = true;
                                    }
                                });
                                if move_to {
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
