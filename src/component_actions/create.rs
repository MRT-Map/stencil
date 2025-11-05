use std::{collections::HashMap, sync::Arc};

use tracing::info;

use crate::{
    App,
    map::MapWindow,
    project::{
        pla3::{PlaComponent, PlaNode, PlaNodeBase},
        skin::SkinType,
    },
};

impl MapWindow {
    pub fn create_point(
        &self,
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        if app.project.new_component_ns.is_empty() {
            return;
        }
        let (Some(cursor_world_pos), Some(skin)) = (self.cursor_world_pos, app.project.skin())
        else {
            return;
        };
        let Some(ty) = self
            .created_point_type
            .as_ref()
            .or_else(|| skin.get_type("simplePoint"))
        else {
            return;
        };
        let Some(style) = ty.point_style_in_zoom_level(self.zoom_level(app)) else {
            return;
        };

        let world_coord = geo::coord! {
            x: cursor_world_pos.x.round() as i32,
            y: cursor_world_pos.y.round() as i32,
        };
        let screen_coord = self.world_to_screen(
            app,
            response.rect.center(),
            geo::coord! { x: world_coord.x as f32, y: world_coord.y as f32 },
        );
        Self::paint_point(ui, response, painter, false, screen_coord, ty.name(), style);

        if !response.clicked_by(egui::PointerButton::Primary) {
            return;
        }
        let component = PlaComponent {
            namespace: app.project.new_component_ns.clone(),
            id: app
                .project
                .components
                .get_new_id(&app.project.new_component_ns),
            ty: Arc::clone(ty),
            display_name: String::new(),
            layer: 0.0,
            nodes: vec![PlaNode::Line {
                coord: world_coord,
                label: None,
            }],
            misc: HashMap::default(),
        };
        info!(?world_coord, %component, "Created new point");

        app.project.components.insert(skin, component);
    }
    pub fn create_line(
        &mut self,
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        if app.project.new_component_ns.is_empty() {
            return;
        }
        let (Some(cursor_world_pos), Some(skin)) = (self.cursor_world_pos, app.project.skin())
        else {
            return;
        };
        let Some(ty) = self
            .created_line_type
            .as_ref()
            .or_else(|| skin.get_type("simpleLine"))
        else {
            return;
        };
        let Some(style) = ty.line_style_in_zoom_level(self.zoom_level(app)) else {
            return;
        };

        let world_coord = geo::coord! {
            x: cursor_world_pos.x.round() as i32,
            y: cursor_world_pos.y.round() as i32,
        };

        match self.created_nodes.last_mut() {
            None => self.created_nodes.push(PlaNode::Line {
                coord: world_coord,
                label: None,
            }),
            Some(
                PlaNode::Line { coord, .. }
                | PlaNode::QuadraticBezier { coord, .. }
                | PlaNode::CubicBezier { coord, .. },
            ) => *coord = world_coord,
        }
        Self::paint_line(
            ui,
            response,
            painter,
            false,
            &self
                .created_nodes
                .iter()
                .map(|a| a.to_screen(app, self, response.rect.center()))
                .collect::<Vec<_>>(),
            style,
        );
        if let Some(curve_vec) = match self.created_nodes.last_chunk::<2>() {
            Some([second_last, PlaNode::QuadraticBezier { ctrl, coord, .. }]) => {
                Some(vec![second_last.coord(), *ctrl, *coord])
            }
            Some(
                [
                    second_last,
                    PlaNode::CubicBezier {
                        ctrl1,
                        ctrl2,
                        coord,
                        ..
                    },
                ],
            ) => Some(vec![second_last.coord(), *ctrl1, *ctrl2, *coord]),
            _ => None,
        } {
            let curve_vec = curve_vec
                .iter()
                .map(|a| {
                    self.world_to_screen(
                        app,
                        response.rect.center(),
                        geo::coord! { x: a.x as f32, y: a.y as f32},
                    )
                })
                .collect::<Vec<_>>();
            painter.add(egui::Shape::dashed_line(
                &curve_vec,
                egui::Stroke::new(4.0, egui::Color32::BLACK),
                8.0,
                8.0,
            ));
            painter.add(egui::Shape::dashed_line(
                &curve_vec,
                egui::Stroke::new(2.0, egui::Color32::WHITE),
                8.0,
                8.0,
            ));
        }

        if response.clicked_by(egui::PointerButton::Secondary) {
            let last_node = self.created_nodes.last_mut().unwrap();
            match *last_node {
                PlaNode::Line { .. } => {
                    self.created_nodes.pop();
                }
                PlaNode::QuadraticBezier { ctrl, coord, label } => {
                    *last_node = PlaNodeBase::Line { coord, label }
                }
                PlaNode::CubicBezier {
                    ctrl1,
                    ctrl2,
                    coord,
                    label,
                } => {
                    *last_node = PlaNode::QuadraticBezier {
                        ctrl: ctrl1,
                        coord,
                        label,
                    }
                }
            }
        } else if response.clicked_by(egui::PointerButton::Primary) {
            if ui.ctx().input(|a| a.modifiers.shift) && self.created_nodes.len() > 1 {
                let last_node = self.created_nodes.last_mut().unwrap();
                match *last_node {
                    PlaNode::Line { coord, label } => {
                        *last_node = PlaNode::QuadraticBezier {
                            ctrl: coord,
                            coord,
                            label,
                        }
                    }
                    PlaNode::QuadraticBezier { ctrl, coord, label } => {
                        *last_node = PlaNode::CubicBezier {
                            ctrl1: ctrl,
                            ctrl2: coord,
                            coord,
                            label,
                        }
                    }
                    _ => {}
                }
            } else {
                self.created_nodes.push(PlaNode::Line {
                    coord: world_coord,
                    label: None,
                });
            }
        }
        if response.double_clicked_by(egui::PointerButton::Primary) {
            self.created_nodes.pop();
            let component = PlaComponent {
                namespace: app.project.new_component_ns.clone(),
                id: app
                    .project
                    .components
                    .get_new_id(&app.project.new_component_ns),
                ty: Arc::clone(ty),
                display_name: String::new(),
                layer: 0.0,
                nodes: self.created_nodes.drain(..).collect(),
                misc: HashMap::default(),
            };
            info!(?component.nodes, %component, "Created new line");

            app.project.components.insert(skin, component);
        }
    }
}
