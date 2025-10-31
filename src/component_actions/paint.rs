use std::{borrow::Cow, sync::Arc};

use geo::{BooleanOps, Buffer, Distance, EuclideanDistance, Intersects, MapCoords};

use crate::{
    App,
    map::{MapWindow, tile_coord::TextureIdResult},
    project::{
        pla3::{PlaComponent, PlaNode, PlaNodeScreen},
        skin::{AreaStyle, LineStyle, PointStyle, SkinType},
    },
};

macro_rules! hovering {
    ($is_hovered:expr, $response:expr, $width:expr, $line:expr) => {
        if !$is_hovered
            && let Some(hover_pos) = $response.hover_pos()
            && geo::Euclidean.distance(&$line, &geo::point! { x: hover_pos.x, y: hover_pos.y })
                < $width / 2.0
        {
            $is_hovered = true;
        }
    };
}

impl MapWindow {
    pub fn paint_components<'a>(
        &mut self,
        app: &'a App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        let mut hovered = None;
        for component in app.project.components.iter() {
            let is_hovered =
                self.paint_component(app, ui, response, painter, hovered.is_none(), component);
            if is_hovered {
                hovered = Some(component);
            }
        }
        if let Some(hovered) = hovered {
            // todo
        }

        self.hovered_component = hovered.map(Arc::clone);
    }
    pub fn paint_component(
        &self,
        app: &App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        component: &PlaComponent,
    ) -> bool {
        let bounding_rect = component.bounding_rect();
        let world_boundaries = self.map_world_boundaries(app, response.rect);
        if world_boundaries.max().x < bounding_rect.min().x
            || bounding_rect.max().x < world_boundaries.min().x
            || world_boundaries.max().y < bounding_rect.min().y
            || bounding_rect.max().y < world_boundaries.min().y
        {
            return false;
        }

        let zl = self.zoom_level(app);
        let mut screen_coords = component
            .nodes
            .iter()
            .map(|a| a.to_screen(app, &self, response.rect.center()));
        match &*component.ty {
            SkinType::Point { styles, .. } => {
                let Some(style) = SkinType::style_in_zoom_level(styles, zl) else {
                    return false;
                };
                let PlaNodeScreen::Line { coord, .. } = screen_coords.next().unwrap() else {
                    unreachable!();
                };
                self.paint_point(ui, response, painter, detect_hovered, coord, style)
            }
            SkinType::Line { styles, .. } => {
                let Some(style) = SkinType::style_in_zoom_level(styles, zl) else {
                    return false;
                };
                self.paint_line(
                    response,
                    painter,
                    detect_hovered,
                    &screen_coords.collect::<Vec<_>>(),
                    style,
                )
            }
            SkinType::Area { styles, .. } => {
                let Some(style) = SkinType::style_in_zoom_level(styles, zl) else {
                    return false;
                };
                self.paint_area(
                    response,
                    painter,
                    detect_hovered,
                    &screen_coords.collect::<Vec<_>>(),
                    style,
                )
            }
        }
    }
    pub fn paint_area(
        &self,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        nodes: &[PlaNodeScreen],
        style: &[AreaStyle],
    ) -> bool {
        let mut is_hovered = !detect_hovered;

        for style in style {
            let AreaStyle::Fill {
                colour,
                outline,
                outline_width,
                ..
            } = style
            else {
                continue;
            };
            let mut previous_coord = Option::<egui::Pos2>::None;

            let mut shapes = Vec::new();
            for node in nodes {
                let final_coord = match *node {
                    PlaNodeScreen::Line { coord, .. } => {
                        if let Some(previous_coord) = previous_coord {
                            let shape = egui::Shape::line_segment(
                                [previous_coord, coord],
                                egui::Stroke::new(*outline_width, outline.unwrap_or_default()),
                            );
                            shapes.push(shape);
                        }
                        coord
                    }
                    PlaNodeScreen::QuadraticBezier { ctrl, coord, .. } => {
                        let shape = egui::epaint::QuadraticBezierShape::from_points_stroke(
                            [previous_coord.unwrap(), ctrl, coord],
                            false,
                            egui::Color32::TRANSPARENT,
                            egui::Stroke::new(*outline_width, outline.unwrap_or_default()),
                        );

                        shapes.push(shape.into());
                        coord
                    }
                    PlaNodeScreen::CubicBezier {
                        ctrl1,
                        ctrl2,
                        coord,
                        ..
                    } => {
                        let shape = egui::epaint::CubicBezierShape::from_points_stroke(
                            [previous_coord.unwrap(), ctrl1, ctrl2, coord],
                            false,
                            egui::Color32::TRANSPARENT,
                            egui::Stroke::new(*outline_width, outline.unwrap_or_default()),
                        );

                        shapes.push(shape.into());
                        coord
                    }
                };

                shapes.push(egui::Shape::circle_filled(
                    final_coord,
                    outline_width / 2.0,
                    colour.unwrap_or_default(),
                ));
                previous_coord = Some(final_coord);
            }

            let polygon = geo::Polygon::new(
                shapes
                    .iter()
                    .flat_map(|a| match a {
                        egui::Shape::LineSegment { points, .. } => vec![
                            geo::coord! { x: points[0].x, y: points[0].y },
                            geo::coord! { x: points[1].x, y: points[1].y },
                        ],
                        egui::Shape::QuadraticBezier(shape) => shape
                            .flatten(None)
                            .into_iter()
                            .map(|a| geo::coord! { x: a.x, y: a.y })
                            .collect(),
                        egui::Shape::CubicBezier(shape) => shape
                            .flatten(None)
                            .into_iter()
                            .map(|a| geo::coord! { x: a.x, y: a.y })
                            .collect(),
                        _ => unreachable!(),
                    })
                    .collect(),
                Vec::new(),
            );

            // let polygon_edge = polygon.difference(&polygon.buffer(16.0 * outline_width));

            hovering!(is_hovered, response, outline_width, polygon);

            let coords = polygon
                .exterior()
                .coords()
                .map(|a| egui::pos2(a.x, a.y))
                .collect::<Vec<_>>();
            painter.add(egui::Shape::convex_polygon(
                coords,
                colour.unwrap_or(egui::Color32::TRANSPARENT),
                egui::Stroke::new(
                    *outline_width,
                    outline.unwrap_or(egui::Color32::TRANSPARENT),
                ),
            ));
        }

        detect_hovered && is_hovered
    }
    pub fn paint_line(
        &self,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        nodes: &[PlaNodeScreen],
        style: &[LineStyle],
    ) -> bool {
        let mut is_hovered = !detect_hovered;

        for style in style {
            let mut previous_coord = Option::<egui::Pos2>::None;
            match style {
                LineStyle::Back {
                    colour,
                    width,
                    unrounded,
                    ..
                }
                | LineStyle::Fore {
                    colour,
                    width,
                    unrounded,
                    ..
                } => {
                    for (i, node) in nodes.iter().enumerate() {
                        let final_coord = match *node {
                            PlaNodeScreen::Line { coord, .. } => {
                                if let Some(previous_coord) = previous_coord {
                                    hovering!(
                                        is_hovered,
                                        response,
                                        width,
                                        geo::Line::new(
                                            geo::coord! { x: previous_coord.x, y: previous_coord.y },
                                            geo::coord! { x: coord.x, y: coord.y },
                                        )
                                    );

                                    painter.line_segment(
                                        [previous_coord, coord],
                                        egui::Stroke::new(*width, colour.unwrap_or_default()),
                                    );
                                }
                                coord
                            }
                            PlaNodeScreen::QuadraticBezier { ctrl, coord, .. } => {
                                let shape = egui::epaint::QuadraticBezierShape::from_points_stroke(
                                    [previous_coord.unwrap(), ctrl, coord],
                                    false,
                                    egui::Color32::TRANSPARENT,
                                    egui::Stroke::new(*width, colour.unwrap_or_default()),
                                );

                                hovering!(
                                    is_hovered,
                                    response,
                                    width,
                                    geo::LineString::new(
                                        shape
                                            .flatten(None)
                                            .into_iter()
                                            .map(|a| geo::coord! { x: a.x, y: a.y })
                                            .collect()
                                    )
                                );

                                painter.add(shape);
                                coord
                            }
                            PlaNodeScreen::CubicBezier {
                                ctrl1,
                                ctrl2,
                                coord,
                                ..
                            } => {
                                let shape = egui::epaint::CubicBezierShape::from_points_stroke(
                                    [previous_coord.unwrap(), ctrl1, ctrl2, coord],
                                    false,
                                    egui::Color32::TRANSPARENT,
                                    egui::Stroke::new(*width, colour.unwrap_or_default()),
                                );

                                hovering!(
                                    is_hovered,
                                    response,
                                    width,
                                    geo::LineString::new(
                                        shape
                                            .flatten(None)
                                            .into_iter()
                                            .map(|a| geo::coord! { x: a.x, y: a.y })
                                            .collect()
                                    )
                                );

                                painter.add(shape);
                                coord
                            }
                        };

                        if !(*unrounded && (i == 0 || i == nodes.len() - 1)) {
                            painter.circle_filled(
                                final_coord,
                                width / 2.0,
                                colour.unwrap_or_default(),
                            );
                        }
                        previous_coord = Some(final_coord);
                    }
                }
                LineStyle::Text { .. } => {}
            }
        }

        detect_hovered && is_hovered
    }
    pub fn paint_point(
        &self,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        coord: egui::Pos2,
        styles: &[PointStyle],
    ) -> bool {
        let mut is_hovered = !detect_hovered;

        for style in styles {
            match style {
                PointStyle::Image {
                    image,
                    size,
                    offset,
                    ..
                } => {
                    let Some(texture_id) = egui::ImageSource::Bytes {
                        uri: Cow::default(),
                        bytes: image.clone().into(),
                    }
                    .load(
                        ui.ctx(),
                        egui::TextureOptions::LINEAR,
                        egui::SizeHint::Scale(2.0.into()),
                    )
                    .ok()
                    .and_then(|a| a.texture_id()) else {
                        continue;
                    };

                    let shape = egui::Shape::image(
                        texture_id,
                        egui::Rect::from_center_size(coord + *offset, *size * 4.0),
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                    if !is_hovered
                        && let Some(hover_pos) = response.hover_pos()
                        && shape.visual_bounding_rect().contains(hover_pos)
                    {
                        is_hovered = true;
                    }
                    painter.add(shape);
                }
                PointStyle::Square {
                    colour,
                    border_radius,
                    size,
                    ..
                } => {
                    let shape = egui::Shape::rect_filled(
                        egui::Rect::from_center_size(coord, egui::Vec2::splat(*size * 4.0)),
                        *border_radius,
                        colour.unwrap_or_default(),
                    );
                    if !is_hovered
                        && let Some(hover_pos) = response.hover_pos()
                        && shape.visual_bounding_rect().contains(hover_pos)
                    {
                        is_hovered = true;
                    }
                    painter.add(shape);
                }
                PointStyle::Text { .. } => {}
            }
        }

        detect_hovered && is_hovered
    }
}
