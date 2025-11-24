use std::{collections::BTreeMap, sync::Arc};

use geo::Vector2DOps;
use itertools::{Either, Itertools};
use tracing::info;

use crate::{
    App,
    component_actions::ComponentEv,
    map::MapWindow,
    project::pla3::{FullId, PlaComponent, PlaNode, PlaNodeBase},
};

const ANGLE_VECTORS: [geo::Coord<f32>; 20] = [
    geo::Coord::<f32> { x: 4.0, y: 0.0 },
    geo::Coord::<f32> { x: 4.0, y: 1.0 },
    geo::Coord::<f32> { x: 3.0, y: 1.0 },
    geo::Coord::<f32> { x: 2.0, y: 1.0 },
    geo::Coord::<f32> { x: 1.5, y: 1.0 },
    geo::Coord::<f32> { x: 1.0, y: 1.0 },
    geo::Coord::<f32> { x: 1.0, y: 1.5 },
    geo::Coord::<f32> { x: 1.0, y: 2.0 },
    geo::Coord::<f32> { x: 1.0, y: 3.0 },
    geo::Coord::<f32> { x: 1.0, y: 4.0 },
    geo::Coord::<f32> { x: 0.0, y: 4.0 },
    geo::Coord::<f32> { x: -1.0, y: 4.0 },
    geo::Coord::<f32> { x: -1.0, y: 3.0 },
    geo::Coord::<f32> { x: -1.0, y: 2.0 },
    geo::Coord::<f32> { x: -1.0, y: 1.5 },
    geo::Coord::<f32> { x: -1.0, y: 1.0 },
    geo::Coord::<f32> { x: -1.5, y: 1.0 },
    geo::Coord::<f32> { x: -2.0, y: 1.0 },
    geo::Coord::<f32> { x: -3.0, y: 1.0 },
    geo::Coord::<f32> { x: -4.0, y: 1.0 },
];

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
        Self::paint_point(
            ui,
            response,
            painter,
            false,
            false,
            screen_coord,
            ty.name(),
            style,
        );

        if !response.clicked_by(egui::PointerButton::Primary) {
            return;
        }
        let component = PlaComponent {
            full_id: FullId::new(
                app.project.new_component_ns.clone(),
                app.project
                    .components
                    .get_new_id(&app.project.new_component_ns),
            ),
            ty: Arc::clone(ty),
            display_name: String::new(),
            layer: 0.0,
            nodes: vec![PlaNode::Line {
                coord: world_coord,
                label: None,
            }],
            misc: BTreeMap::default(),
        };
        info!(?world_coord, %component, "Created new point");
        app.run_event(ComponentEv::Create(vec![component]), ui.ctx());
    }
    #[inline]
    pub fn create_line(
        &mut self,
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        self.create_line_or_area::<true>(app, ui, response, painter);
    }
    #[inline]
    pub fn create_area(
        &mut self,
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        self.create_line_or_area::<false>(app, ui, response, painter);
    }
    pub fn create_line_or_area<const IS_LINE: bool>(
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
        let (ty, style) = if IS_LINE {
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
            (Either::Left(ty), Either::Left(style))
        } else {
            let Some(ty) = self
                .created_area_type
                .as_ref()
                .or_else(|| skin.get_type("simpleArea"))
            else {
                return;
            };

            let Some(style) = ty.area_style_in_zoom_level(self.zoom_level(app)) else {
                return;
            };
            (Either::Right(ty), Either::Right(style))
        };

        let mut world_coord = geo::coord! {
            x: cursor_world_pos.x.round() as i32,
            y: cursor_world_pos.y.round() as i32,
        };

        if ui.ctx().input(|a| a.modifiers.alt)
            && let Some(prev_coord) = match self.created_nodes.last() {
                Some(PlaNode::Line { .. }) => self
                    .created_nodes
                    .get(self.created_nodes.len() - 2)
                    .map(|a| a.coord()),
                Some(PlaNode::QuadraticBezier { ctrl, .. }) => Some(*ctrl),
                Some(PlaNodeBase::CubicBezier { ctrl2, .. }) => Some(*ctrl2),
                None => None,
            }
            && world_coord != prev_coord
        {
            let angle_vec = {
                let c = world_coord - prev_coord;
                geo::coord! {
                    x: c.x as f32,
                    y: c.y as f32
                }
            };
            let (closest_angle_vec, _) = ANGLE_VECTORS
                .into_iter()
                .chain(ANGLE_VECTORS.into_iter().map(|a| -a))
                .map(|v| {
                    (
                        v,
                        v.try_normalize()
                            .unwrap()
                            .dot_product(angle_vec.try_normalize().unwrap()),
                    )
                })
                .sorted_by(|(_, k1), (_, k2)| k1.total_cmp(k2))
                .next()
                .unwrap();
            // adapted from https://docs.rs/glam/latest/src/glam/f32/vec2.rs.html#618-622
            let world_coord_f32 = closest_angle_vec * angle_vec.dot_product(closest_angle_vec)
                / closest_angle_vec.dot_product(closest_angle_vec);
            world_coord = geo::coord! {
                x: world_coord_f32.x.round() as i32,
                y: world_coord_f32.y.round() as i32,
            } + prev_coord;
        }

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

        let screen_nodes = self
            .created_nodes
            .iter()
            .map(|a| a.to_screen(app, self, response.rect.center()))
            .collect::<Vec<_>>();
        match style {
            Either::Left(style) => {
                Self::paint_line(response, painter, false, false, &screen_nodes, style);
            }
            Either::Right(style) => {
                Self::paint_area(response, painter, false, false, &screen_nodes, style);
            }
        }

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
            Some(
                [
                    PlaNode::Line { coord: coord1, .. },
                    PlaNode::Line { coord: coord2, .. },
                ],
            ) => (!IS_LINE && self.created_nodes.len() == 2).then_some(vec![*coord1, *coord2]),
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
            info!(?last_node, "Undoing last control point / node");
            match *last_node {
                PlaNode::Line { .. } => {
                    self.created_nodes.pop();
                }
                PlaNode::QuadraticBezier { coord, label, .. } => {
                    *last_node = PlaNodeBase::Line { coord, label }
                }
                PlaNode::CubicBezier {
                    ctrl1,
                    coord,
                    label,
                    ..
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
                    PlaNode::CubicBezier { .. } => {}
                }
                info!(?last_node, "Adding control point");
            } else if self
                .created_nodes
                .last_chunk::<2>()
                .is_none_or(|[sl, l]| sl.coord() != l.coord())
            {
                self.created_nodes.push(PlaNode::Line {
                    coord: world_coord,
                    label: None,
                });
                info!(?world_coord, "Adding node");
            }
        }
        if response.double_clicked_by(egui::PointerButton::Primary)
            || response.double_clicked_by(egui::PointerButton::Middle)
        {
            self.created_nodes.pop();
            if self.created_nodes.len() >= (if IS_LINE { 2 } else { 3 }) {
                if !IS_LINE
                    && self.created_nodes.first().unwrap().coord()
                        != self.created_nodes.last().unwrap().coord()
                {
                    self.created_nodes.push(PlaNode::Line {
                        coord: self.created_nodes.first().unwrap().coord(),
                        label: None,
                    });
                }
                let component = PlaComponent {
                    full_id: FullId::new(
                        app.project.new_component_ns.clone(),
                        app.project
                            .components
                            .get_new_id(&app.project.new_component_ns),
                    ),
                    ty: Arc::clone(ty.into_inner()),
                    display_name: String::new(),
                    layer: 0.0,
                    nodes: self.created_nodes.drain(..).collect(),
                    misc: BTreeMap::default(),
                };
                info!(?component.nodes, %component, "Created new {}", if IS_LINE {"line"} else {"area"});
                app.run_event(ComponentEv::Create(vec![component]), ui.ctx());
            } else {
                self.created_nodes.clear();
                info!(
                    "No new {} created due to too few points",
                    if IS_LINE { "line" } else { "area" }
                );
            }
        }
    }
}
