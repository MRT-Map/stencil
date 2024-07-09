use bevy::prelude::{Event, ResMut, Trigger};
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;

use crate::ui::panel::dock::{
    window_action_handler, DockWindow, PanelDockState, PanelParams, TabViewer,
};

#[derive(Clone, Copy)]
pub struct ComponentList;

#[derive(Clone, Copy, Event)]
pub struct OpenComponentListAct;

impl DockWindow for ComponentList {
    fn title(self) -> String {
        "Component List".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams {
            queries, camera, ..
        } = tab_viewer.params;
        let mut transform = camera.single_mut();
        let query = queries.p1();
        let components = query.iter().into_group_map_by(|a| a.namespace.to_owned());
        for (ns, components) in components {
            ui.collapsing(&ns, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::auto().at_least(150.0))
                    .column(Column::auto().at_least(100.0))
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
                        for component in components {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(component.to_string());
                                });
                                row.col(|ui| {
                                    ui.label(&component.ty);
                                });
                                row.col(|ui| {
                                    if let Some(coords) = component.nodes.first() {
                                        if ui.small_button("Move to").clicked() {
                                            transform.translation.x = coords.0.x as f32;
                                            transform.translation.y = coords.0.y as f32;
                                        }
                                    }
                                });
                            });
                        }
                    });
            });
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn on_component_list(
    _trigger: Trigger<OpenComponentListAct>,
    mut state: ResMut<PanelDockState>,
) {
    window_action_handler(&mut state, ComponentList);
}
