use std::{borrow::Cow, f32::consts::FRAC_PI_2};

use geo::{
    Area, BooleanOps, Buffer, Contains, CoordsIter, Distance, TriangulateDelaunay,
    buffer::{BufferStyle, LineJoin},
    triangulate_delaunay::{DelaunayTriangulationConfig, TriangulationResult},
};
use itertools::Itertools;
use tracing::{debug, error};

use crate::{
    App,
    coord_conversion::CoordConversionExt,
    map::MapWindow,
    project::{
        pla3::{PlaComponent, PlaNodeScreen, PlaNodeScreenVec},
        skin::{AreaStyle, LineStyle, PointStyle, SkinType},
    },
};

pub const TOLERANCE: Option<f32> = Some(1.0);

macro_rules! hovering {
    ($is_hovered:expr, $response:expr, $width:expr, $line:expr) => {
        if !$is_hovered
            && let Some(hover_pos) = $response.hover_pos()
            && geo::Euclidean.distance(&$line, &geo::point! { x: hover_pos.x, y: hover_pos.y })
                < $width / 2.0 * 1.5
        {
            $is_hovered = true;
        }
    };
}
#[must_use]
pub struct PaintResult<'a> {
    pub shapes: Vec<egui::Shape>,
    pub is_hovering: bool,
    pub screen_coords: PlaNodeScreenVec,
    pub point_style: Option<&'a [PointStyle]>,
}
impl PaintResult<'_> {
    pub fn paint(self, painter: &egui::Painter) {
        painter.add(self.shapes);
    }
}
#[must_use]
pub struct PartialPaintResult {
    pub shapes: Vec<egui::Shape>,
    pub is_hovering: bool,
}
impl PartialPaintResult {
    pub fn paint(self, painter: &egui::Painter) {
        painter.add(self.shapes);
    }
}

impl MapWindow {
    pub fn paint_components(app: &mut App, response: &egui::Response, painter: &egui::Painter) {
        let mut hovered_component = None;
        let mut selected_shapes = Vec::new();
        let mut all_shapes = Vec::new();
        for component in app.project.components.iter().rev() {
            let is_selected = app.ui.map.is_selected(&component.full_id);
            let Some(PaintResult {
                is_hovering,
                screen_coords,
                point_style,
                shapes,
            }) = Self::paint_component(
                app,
                response,
                hovered_component.is_none(),
                is_selected,
                component,
            )
            else {
                continue;
            };

            all_shapes.push(shapes.into());

            if app.mode.is_editing() {
                continue;
            }

            if is_hovering {
                hovered_component = Some((
                    component.full_id.clone(),
                    Self::white_dash(
                        &Self::outline(point_style, &screen_coords),
                        matches!(&*component.ty, SkinType::Line { .. }),
                    ),
                ));
            }
            if is_selected {
                selected_shapes.extend(Self::select_dash(
                    &Self::outline(point_style, &screen_coords),
                    matches!(&*component.ty, SkinType::Line { .. }),
                ));
            }
        }

        all_shapes.reverse();
        painter.add(all_shapes);
        painter.add(selected_shapes);

        let hovered_component = if let Some((id, hover_shapes)) = hovered_component {
            painter.add(hover_shapes);
            Some(id)
        } else {
            None
        };

        match (&app.ui.map.hovered_component, &hovered_component) {
            (Some(id), None) => {
                debug!(%id, "Mouse out");
            }
            (None, Some(id)) => {
                debug!(%id, "Mouse over");
            }
            _ => {}
        }
        app.ui.map.hovered_component = hovered_component;
    }
    pub fn paint_component<'a>(
        app: &App,
        response: &egui::Response,
        detect_hovered: bool,
        is_selected: bool,
        component: &'a PlaComponent,
    ) -> Option<PaintResult<'a>> {
        let bounding_rect = component.bounding_rect();
        let world_boundaries = app.map_world_boundaries(response.rect);
        if world_boundaries.max().x < bounding_rect.min().x
            || bounding_rect.max().x < world_boundaries.min().x
            || world_boundaries.max().y < bounding_rect.min().y
            || bounding_rect.max().y < world_boundaries.min().y
        {
            return None;
        }

        let zl = app.map_zoom_level();
        let screen_coords = if is_selected && let Some(move_delta) = app.ui.map.comp_move_delta() {
            Cow::Owned(component.nodes.clone() + move_delta)
        } else {
            Cow::Borrowed(&component.nodes)
        }
        .to_screen(app, response.rect.center());
        let (result, point_style) = match &*component.ty {
            SkinType::Point {
                styles,
                name: style_name,
                ..
            } => {
                let style = SkinType::style_in_zoom_level(styles, zl)?;
                let PlaNodeScreen::Line { coord, .. } = screen_coords[0] else {
                    unreachable!();
                };
                (
                    Self::paint_point(response, detect_hovered, coord, style_name, style),
                    Some(style.as_ref()),
                )
            }
            SkinType::Line { styles, .. } => {
                let style = SkinType::style_in_zoom_level(styles, zl)?;
                (
                    Self::paint_line(response, detect_hovered, &screen_coords, style),
                    None,
                )
            }
            SkinType::Area { styles, .. } => {
                let style = SkinType::style_in_zoom_level(styles, zl)?;
                (
                    Self::paint_area(response, detect_hovered, &screen_coords, style),
                    None,
                )
            }
        };
        Some(PaintResult {
            is_hovering: result.is_hovering,
            shapes: result.shapes,
            screen_coords,
            point_style,
        })
    }

    fn outline(point_style: Option<&[PointStyle]>, nodes: &PlaNodeScreenVec) -> Vec<egui::Pos2> {
        let Some(style) = point_style else {
            return nodes.outline();
        };
        let PlaNodeScreen::Line { coord, .. } = nodes[0] else {
            unreachable!();
        };
        let dimensions = style
            .iter()
            .filter_map(|a| match a {
                PointStyle::Image { size, .. } => Some(*size),
                PointStyle::Square { size, .. } => Some(egui::Vec2::splat(*size)),
                PointStyle::Text { .. } => None,
            })
            .reduce(egui::Vec2::max)
            .unwrap_or_else(|| egui::Vec2::splat(8.0));
        vec![
            coord + 2.0 * egui::vec2(dimensions.x, dimensions.y),
            coord + 2.0 * egui::vec2(dimensions.x, -dimensions.y),
            coord + 2.0 * egui::vec2(-dimensions.x, -dimensions.y),
            coord + 2.0 * egui::vec2(-dimensions.x, dimensions.y),
            coord + 2.0 * egui::vec2(dimensions.x, dimensions.y),
        ]
    }

    // adapted from egui::Painter::arrow
    fn arrow(
        origin: egui::Pos2,
        tip: egui::Pos2,
        tip_length: f32,
        stroke: egui::Stroke,
    ) -> Vec<egui::Shape> {
        let rot = egui::emath::Rot2::from_angle(std::f32::consts::TAU / 10.0);
        let dir = (tip - origin).normalized();
        vec![
            egui::Shape::line_segment([origin, tip], stroke),
            egui::Shape::line_segment([tip, tip - tip_length * (rot * dir)], stroke),
            egui::Shape::line_segment([tip, tip - tip_length * (rot.inverse() * dir)], stroke),
        ]
    }
    fn add_arrows(dashes: Vec<egui::Shape>) -> Vec<egui::Shape> {
        dashes
            .into_iter()
            .circular_tuple_windows()
            .flat_map(|(shape1, shape2)| {
                let egui::Shape::LineSegment { points, stroke } = shape1 else {
                    unreachable!()
                };
                let egui::Shape::LineSegment {
                    points: points2, ..
                } = shape2
                else {
                    unreachable!()
                };
                if points[1] == points2[0] {
                    return vec![shape1];
                }
                Self::arrow(points[0], points[1], 4.0, stroke)
            })
            .collect()
    }

    fn dash(path: &[egui::Pos2], colour: egui::Color32, arrows: bool) -> Vec<egui::Shape> {
        let mut dashes = egui::Shape::dashed_line(
            path,
            egui::Stroke::new(6.0_f32, egui::Color32::BLACK),
            8.0,
            8.0,
        );
        if arrows {
            dashes = Self::add_arrows(dashes);
        }

        let mut dashes2 =
            egui::Shape::dashed_line(path, egui::Stroke::new(2.0_f32, colour), 8.0, 8.0);
        if arrows {
            dashes2 = Self::add_arrows(dashes2);
        }

        dashes.extend(dashes2);
        dashes
    }
    pub fn white_dash(path: &[egui::Pos2], arrows: bool) -> Vec<egui::Shape> {
        Self::dash(path, egui::Color32::WHITE, arrows)
    }

    pub fn select_dash(path: &[egui::Pos2], arrows: bool) -> Vec<egui::Shape> {
        Self::dash(path, egui::Color32::YELLOW, arrows)
    }
    fn image_shape_from_bytes(
        ctx: &egui::Context,
        uri: impl Into<Cow<'static, str>>,
        bytes: impl Into<egui::load::Bytes>,
        rect: egui::Rect,
    ) -> Option<egui::Shape> {
        let texture_id = egui::ImageSource::Bytes {
            uri: uri.into(),
            bytes: bytes.into(),
        }
        .load(
            ctx,
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
        response: &egui::Response,
        detect_hovered: bool,
        nodes: &PlaNodeScreenVec,
        style: &[AreaStyle],
    ) -> PartialPaintResult {
        let mut is_hovered = !detect_hovered;
        let mut shapes = Vec::new();

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

            let mut outline_shapes = Vec::new();
            for node in nodes {
                let final_coord = match *node {
                    PlaNodeScreen::Line { coord, .. } => {
                        if let Some(previous_coord) = previous_coord {
                            let shape = egui::Shape::line_segment(
                                [previous_coord, coord],
                                egui::Stroke::new(
                                    *outline_width * 4.0,
                                    outline.unwrap_or_default(),
                                ),
                            );
                            outline_shapes.push(shape);
                        }
                        coord
                    }
                    PlaNodeScreen::QuadraticBezier { ctrl, coord, .. } => {
                        let shape = egui::epaint::QuadraticBezierShape::from_points_stroke(
                            [previous_coord.unwrap(), ctrl, coord],
                            false,
                            egui::Color32::TRANSPARENT,
                            egui::Stroke::new(*outline_width * 4.0, outline.unwrap_or_default()),
                        );

                        outline_shapes.push(shape.into());
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
                            egui::Stroke::new(*outline_width * 4.0, outline.unwrap_or_default()),
                        );

                        outline_shapes.push(shape.into());
                        coord
                    }
                };

                outline_shapes.push(egui::Shape::circle_filled(
                    final_coord,
                    *outline_width,
                    colour.unwrap_or_default(),
                ));
                previous_coord = Some(final_coord);
            }

            let polygon = geo::Polygon::new(
                outline_shapes
                    .iter()
                    .flat_map(|a| match a {
                        egui::Shape::LineSegment { points, .. } => {
                            vec![points[0].to_geo_coord_f32(), points[1].to_geo_coord_f32()]
                        }
                        egui::Shape::QuadraticBezier(shape) => shape
                            .flatten(TOLERANCE)
                            .into_iter()
                            .map(CoordConversionExt::to_geo_coord_f32)
                            .collect(),
                        egui::Shape::CubicBezier(shape) => shape
                            .flatten(TOLERANCE)
                            .into_iter()
                            .map(CoordConversionExt::to_geo_coord_f32)
                            .collect(),
                        egui::Shape::Circle(_) => Vec::new(),
                        _ => unreachable!(),
                    })
                    .collect(),
                Vec::new(),
            );

            let polygon_edge = if polygon.coords_count() < 2 {
                None
            } else {
                let buffer = polygon.buffer_with_style(
                    BufferStyle::new(-100.0).line_join(LineJoin::Miter(FRAC_PI_2)),
                );
                if buffer.signed_area() < f32::EPSILON {
                    None
                } else {
                    Some(polygon.difference(&buffer))
                }
            };
            if !is_hovered
                && let Some(hover_pos) = response.hover_pos()
                && polygon_edge.as_ref().map_or_else(
                    || polygon.contains(&hover_pos.to_geo_coord_f32()),
                    |polygon_edge| polygon_edge.contains(&hover_pos.to_geo_coord_f32()),
                )
            {
                is_hovered = true;
            }

            #[expect(clippy::items_after_statements)]
            fn triangulate<'a>(
                p: &'a impl TriangulateDelaunay<'a, f32>,
            ) -> TriangulationResult<Vec<[egui::Pos2; 3]>> {
                p.constrained_triangulation(DelaunayTriangulationConfig::default())
                    .map(|a| {
                        a.iter()
                            .map(|a| [a.0, a.1, a.2].map(CoordConversionExt::to_egui_pos2))
                            .collect::<Vec<_>>()
                    })
            }

            if polygon_edge.is_some()
                && let Ok(fill_triangles) = triangulate(&polygon)
            {
                let fill_colour =
                    colour.map_or(egui::Color32::TRANSPARENT, |c| c.gamma_multiply(0.5));
                for triangle in fill_triangles {
                    shapes.push(egui::Shape::convex_polygon(
                        triangle.to_vec(),
                        fill_colour,
                        egui::Stroke::default(),
                    ));
                }
            }
            if let Ok(edge_triangles) =
                polygon_edge.map_or_else(|| triangulate(&polygon), |p| triangulate(&p))
            {
                let edge_colour = colour.unwrap_or(egui::Color32::TRANSPARENT);
                for triangle in edge_triangles {
                    shapes.push(egui::Shape::convex_polygon(
                        triangle.to_vec(),
                        edge_colour,
                        egui::Stroke::new(outline_width.max(1.0), edge_colour),
                    ));
                }
            }

            shapes.push(outline_shapes.into());
        }

        PartialPaintResult {
            is_hovering: detect_hovered && is_hovered,
            shapes,
        }
    }
    pub fn paint_line(
        response: &egui::Response,
        detect_hovered: bool,
        nodes: &PlaNodeScreenVec,
        style: &[LineStyle],
    ) -> PartialPaintResult {
        let mut is_hovered = !detect_hovered;
        let mut shapes = Vec::new();

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
                                            previous_coord.to_geo_coord_f32(),
                                            coord.to_geo_coord_f32(),
                                        )
                                    );

                                    shapes.push(egui::Shape::line_segment(
                                        [previous_coord, coord],
                                        egui::Stroke::new(width, colour.unwrap_or_default()),
                                    ));
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
                                    .flatten(TOLERANCE)
                                    .into_iter()
                                    .map(CoordConversionExt::to_geo_coord_f32)
                                    .collect::<Vec<_>>();
                                hovering!(
                                    is_hovered,
                                    response,
                                    width,
                                    geo::LineString::new(approx)
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
                                    egui::Stroke::new(width, colour.unwrap_or_default()),
                                );

                                let approx = shape
                                    .flatten(TOLERANCE)
                                    .into_iter()
                                    .map(CoordConversionExt::to_geo_coord_f32)
                                    .collect::<Vec<_>>();
                                hovering!(
                                    is_hovered,
                                    response,
                                    width,
                                    geo::LineString::new(approx)
                                );

                                shapes.push(shape.into());
                                coord
                            }
                        };

                        if !(*unrounded && (i == 0 || i == nodes.len() - 1)) {
                            shapes.push(
                                egui::epaint::CircleShape::filled(
                                    final_coord,
                                    width / 2.0,
                                    colour.unwrap_or_default(),
                                )
                                .into(),
                            );
                        }

                        previous_coord = Some(final_coord);
                    }
                }
                LineStyle::Text { .. } => {}
            }
        }

        PartialPaintResult {
            is_hovering: detect_hovered && is_hovered,
            shapes,
        }
    }
    pub fn paint_point(
        response: &egui::Response,
        detect_hovered: bool,
        coord: egui::Pos2,
        style_name: &str,
        style: &[PointStyle],
    ) -> PartialPaintResult {
        let mut is_hovered = !detect_hovered;
        let mut shapes = Vec::new();

        for style in style {
            match style {
                PointStyle::Image {
                    image,
                    size,
                    offset,
                    extension,
                    ..
                } => {
                    let Some(shape) = Self::image_shape_from_bytes(
                        &response.ctx,
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
                    shapes.push(shape);
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
                    shapes.push(shape);
                }
                PointStyle::Text { .. } => {}
            }
        }

        PartialPaintResult {
            is_hovering: detect_hovered && is_hovered,
            shapes,
        }
    }
}
