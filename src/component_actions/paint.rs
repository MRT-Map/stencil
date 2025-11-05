use std::{borrow::Cow, sync::Arc};

use geo::Distance;
use tracing::error;

use crate::{
    App,
    map::MapWindow,
    mode::EditorMode,
    project::{
        pla3::{PlaComponent, PlaNodeScreen},
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
    pub fn paint_components(
        &mut self,
        app: &App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        let mut hovered_shape = [EditorMode::Select, EditorMode::Nodes]
            .contains(&app.mode)
            .then_some(None);
        self.hovered_component = None;
        for component in app.project.components.iter() {
            let shape = self.paint_component(
                app,
                ui,
                response,
                painter,
                hovered_shape.as_ref().is_none_or(Option::is_none),
                component,
            );
            if shape.is_some()
                && let Some(hovered_shape) = &mut hovered_shape
            {
                self.hovered_component = Some(Arc::clone(component));
                *hovered_shape = shape;
            }
        }
        if let Some(Some(hovered_shape)) = hovered_shape {
            painter.add(hovered_shape);
            ui.ctx().request_repaint_after_secs(0.5);
        }
    }
    pub fn paint_component(
        &self,
        app: &App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        component: &PlaComponent,
    ) -> Option<Vec<egui::Shape>> {
        let bounding_rect = component.bounding_rect();
        let world_boundaries = self.map_world_boundaries(app, response.rect);
        if world_boundaries.max().x < bounding_rect.min().x
            || bounding_rect.max().x < world_boundaries.min().x
            || world_boundaries.max().y < bounding_rect.min().y
            || bounding_rect.max().y < world_boundaries.min().y
        {
            return None;
        }

        let zl = self.zoom_level(app);
        let mut screen_coords = component
            .nodes
            .iter()
            .map(|a| a.to_screen(app, self, response.rect.center()));
        match &*component.ty {
            SkinType::Point {
                styles,
                name: style_name,
                ..
            } => {
                let style = SkinType::style_in_zoom_level(styles, zl)?;
                let PlaNodeScreen::Line { coord, .. } = screen_coords.next().unwrap() else {
                    unreachable!();
                };
                Self::paint_point(
                    ui,
                    response,
                    painter,
                    detect_hovered,
                    coord,
                    style_name,
                    style,
                )
            }
            SkinType::Line { styles, .. } => {
                let style = SkinType::style_in_zoom_level(styles, zl)?;
                Self::paint_line(
                    ui,
                    response,
                    painter,
                    detect_hovered,
                    &screen_coords.collect::<Vec<_>>(),
                    style,
                )
            }
            SkinType::Area { styles, .. } => {
                let style = SkinType::style_in_zoom_level(styles, zl)?;
                Self::paint_area(
                    ui,
                    response,
                    painter,
                    detect_hovered,
                    &screen_coords.collect::<Vec<_>>(),
                    style,
                )
            }
        }
    }

    fn hover_dash(ui: &egui::Ui, path: &[egui::Pos2]) -> Vec<egui::Shape> {
        let mut dashes = egui::Shape::dashed_line_with_offset(
            path,
            egui::Stroke::new(6.0, egui::Color32::BLACK),
            &[8.0],
            &[8.0],
            ui.ctx().input(|a| a.time * 4.0 % 16.0) as f32,
        );
        dashes.extend(egui::Shape::dashed_line_with_offset(
            path,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
            &[8.0],
            &[8.0],
            ui.ctx().input(|a| a.time * 4.0 % 16.0) as f32,
        ));
        dashes
    }
    fn image_shape_from_bytes(
        ui: &egui::Ui,
        uri: impl Into<Cow<'static, str>>,
        bytes: impl Into<egui::load::Bytes>,
        rect: egui::Rect,
    ) -> Option<egui::Shape> {
        let texture_id = egui::ImageSource::Bytes {
            uri: uri.into(),
            bytes: bytes.into(),
        }
        .load(
            ui.ctx(),
            egui::TextureOptions::LINEAR,
            egui::SizeHint::Scale(2.0.into()),
        )
        .inspect_err(|e| error!("{e:?}"))
        .ok()
        .and_then(|a| a.texture_id())?;

        Some(egui::Shape::image(
            texture_id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        ))
    }

    pub fn paint_area(
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        nodes: &[PlaNodeScreen],
        style: &[AreaStyle],
    ) -> Option<Vec<egui::Shape>> {
        let mut is_hovered = !detect_hovered;

        let mut hover_coords = Vec::new();
        let mut hover_coords_is_filled = !detect_hovered;

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
                            if !hover_coords_is_filled {
                                hover_coords.extend([previous_coord, coord]);
                            }
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

                        if !hover_coords_is_filled {
                            hover_coords.extend(
                                shape
                                    .flatten(Some(0.1))
                                    .iter()
                                    .map(|a| egui::pos2(a.x, a.y)),
                            );
                        }
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

                        if !hover_coords_is_filled {
                            hover_coords.extend(
                                shape
                                    .flatten(Some(0.1))
                                    .iter()
                                    .map(|a| egui::pos2(a.x, a.y)),
                            );
                        }
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
            hover_coords_is_filled = true;

            let polygon = geo::Polygon::new(
                shapes
                    .iter()
                    .flat_map(|a| match a {
                        egui::Shape::LineSegment { points, .. } => vec![
                            geo::coord! { x: points[0].x, y: points[0].y },
                            geo::coord! { x: points[1].x, y: points[1].y },
                        ],
                        egui::Shape::QuadraticBezier(shape) => shape
                            .flatten(Some(0.1))
                            .into_iter()
                            .map(|a| geo::coord! { x: a.x, y: a.y })
                            .collect(),
                        egui::Shape::CubicBezier(shape) => shape
                            .flatten(Some(0.1))
                            .into_iter()
                            .map(|a| geo::coord! { x: a.x, y: a.y })
                            .collect(),
                        egui::Shape::Circle(_) => Vec::new(),
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

        (detect_hovered && is_hovered).then(|| Self::hover_dash(ui, &hover_coords))
    }
    pub fn paint_line(
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        nodes: &[PlaNodeScreen],
        style: &[LineStyle],
    ) -> Option<Vec<egui::Shape>> {
        let mut is_hovered = !detect_hovered;

        let mut hover_coords = Vec::new();
        let mut hover_coords_is_filled = !detect_hovered;

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
                    let width = 2.0 * width;
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
                                        egui::Stroke::new(width, colour.unwrap_or_default()),
                                    );
                                    if !hover_coords_is_filled {
                                        hover_coords.extend([previous_coord, coord]);
                                    }
                                }
                                coord
                            }
                            PlaNodeScreen::QuadraticBezier { ctrl, coord, .. } => {
                                let shape = egui::epaint::QuadraticBezierShape::from_points_stroke(
                                    [previous_coord.unwrap(), ctrl, coord],
                                    false,
                                    egui::Color32::TRANSPARENT,
                                    egui::Stroke::new(width, colour.unwrap_or_default()),
                                );

                                let approx = shape
                                    .flatten(Some(0.1))
                                    .into_iter()
                                    .map(|a| geo::coord! { x: a.x, y: a.y })
                                    .collect::<Vec<_>>();
                                if !hover_coords_is_filled {
                                    hover_coords
                                        .extend(approx.iter().map(|a| egui::pos2(a.x, a.y)));
                                }
                                hovering!(
                                    is_hovered,
                                    response,
                                    width,
                                    geo::LineString::new(approx)
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
                                    egui::Stroke::new(width, colour.unwrap_or_default()),
                                );

                                let approx = shape
                                    .flatten(Some(0.1))
                                    .into_iter()
                                    .map(|a| geo::coord! { x: a.x, y: a.y })
                                    .collect::<Vec<_>>();
                                if !hover_coords_is_filled {
                                    hover_coords
                                        .extend(approx.iter().map(|a| egui::pos2(a.x, a.y)));
                                }
                                hovering!(
                                    is_hovered,
                                    response,
                                    width,
                                    geo::LineString::new(approx)
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
                    hover_coords_is_filled = true;
                }
                LineStyle::Text { .. } => {}
            }
        }

        (detect_hovered && is_hovered).then(|| Self::hover_dash(ui, &hover_coords))
    }
    pub fn paint_point(
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        coord: egui::Pos2,
        style_name: &str,
        styles: &[PointStyle],
    ) -> Option<Vec<egui::Shape>> {
        let mut is_hovered = !detect_hovered;

        for style in styles {
            match style {
                PointStyle::Image {
                    image,
                    size,
                    offset,
                    extension,
                    ..
                } => {
                    let Some(shape) = Self::image_shape_from_bytes(
                        ui,
                        format!(
                            "{style_name}.{}",
                            if extension == "svg+xml" {
                                "svg"
                            } else {
                                &extension
                            }
                        ),
                        image.clone(),
                        egui::Rect::from_center_size(coord + *offset, *size * 4.0),
                    ) else {
                        continue;
                    };
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

        (detect_hovered && is_hovered).then(|| {
            let dimensions = styles
                .iter()
                .filter_map(|a| match a {
                    PointStyle::Image { size, .. } => Some(*size),
                    PointStyle::Square { size, .. } => Some(egui::Vec2::splat(*size)),
                    PointStyle::Text { .. } => None,
                })
                .reduce(egui::Vec2::max)
                .unwrap_or_else(|| egui::Vec2::splat(8.0));

            Self::hover_dash(
                ui,
                &[
                    coord + 2.0 * egui::vec2(dimensions.x, dimensions.y),
                    coord + 2.0 * egui::vec2(dimensions.x, -dimensions.y),
                    coord + 2.0 * egui::vec2(-dimensions.x, -dimensions.y),
                    coord + 2.0 * egui::vec2(-dimensions.x, dimensions.y),
                ],
            )
        })
    }
}
