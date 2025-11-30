use std::borrow::Cow;

use geo::{Contains, Distance};
use itertools::Itertools;
use tracing::error;

use crate::{
    App,
    coord_conversion::CoordConversionExt,
    map::MapWindow,
    project::{
        pla3::{PlaComponent, PlaNodeScreen},
        skin::{AreaStyle, LineStyle, PointStyle, SkinType},
    },
};

pub const TOLERANCE: Option<f32> = Some(0.1);

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
pub enum PaintResult {
    None,
    Hovered(Vec<egui::Pos2>),
    Selected(Vec<egui::Pos2>),
    HoveredAndSelected(Vec<egui::Pos2>),
}
impl PaintResult {
    pub fn from_conditions(
        is_selected: bool,
        detect_hovered: bool,
        is_hovered: bool,
        hover_coords: Vec<egui::Pos2>,
    ) -> Self {
        if is_selected && detect_hovered && is_hovered {
            Self::HoveredAndSelected(hover_coords)
        } else if is_selected {
            Self::Selected(hover_coords)
        } else if detect_hovered && is_hovered {
            Self::Hovered(hover_coords)
        } else {
            Self::None
        }
    }
}

impl MapWindow {
    pub fn paint_components(
        app: &mut App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        let mut hovered_shapes = Vec::new();
        app.ui.map.hovered_component = None;
        for component in app.project.components.iter() {
            let result = Self::paint_component(
                app,
                ui,
                response,
                painter,
                app.ui.map.hovered_component.is_none(),
                app.ui
                    .map
                    .selected_components
                    .iter()
                    .any(|a| a == &component.full_id),
                component,
            );

            if !app.mode.is_editing() {
                match result {
                    PaintResult::Hovered(path) | PaintResult::HoveredAndSelected(path) => {
                        app.ui.map.hovered_component = Some(component.full_id.clone());
                        hovered_shapes.extend(Self::hover_dash(
                            &path,
                            matches!(&*component.ty, SkinType::Line { .. }),
                        ));
                    }
                    PaintResult::Selected(path) => {
                        hovered_shapes.extend(Self::select_dash(
                            &path,
                            matches!(&*component.ty, SkinType::Line { .. }),
                        ));
                    }
                    PaintResult::None => {}
                }
            }
        }
        painter.add(hovered_shapes);
    }
    pub fn paint_component(
        app: &App,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        is_selected: bool,
        component: &PlaComponent,
    ) -> PaintResult {
        let bounding_rect = component.bounding_rect();
        let world_boundaries = app.map_world_boundaries(response.rect);
        if world_boundaries.max().x < bounding_rect.min().x
            || bounding_rect.max().x < world_boundaries.min().x
            || world_boundaries.max().y < bounding_rect.min().y
            || bounding_rect.max().y < world_boundaries.min().y
        {
            return PaintResult::None;
        }

        let zl = app.map_zoom_level();
        let mut screen_coords = component
            .nodes
            .iter()
            .map(|a| a.to_screen(app, response.rect.center()));
        match &*component.ty {
            SkinType::Point {
                styles,
                name: style_name,
                ..
            } => {
                let Some(style) = SkinType::style_in_zoom_level(styles, zl) else {
                    return PaintResult::None;
                };
                let PlaNodeScreen::Line { coord, .. } = screen_coords.next().unwrap() else {
                    unreachable!();
                };
                Self::paint_point(
                    ui,
                    response,
                    painter,
                    detect_hovered,
                    is_selected,
                    coord,
                    style_name,
                    style,
                )
            }
            SkinType::Line { styles, .. } => {
                let Some(style) = SkinType::style_in_zoom_level(styles, zl) else {
                    return PaintResult::None;
                };
                Self::paint_line(
                    response,
                    painter,
                    detect_hovered,
                    is_selected,
                    &screen_coords.collect::<Vec<_>>(),
                    style,
                )
            }
            SkinType::Area { styles, .. } => {
                let Some(style) = SkinType::style_in_zoom_level(styles, zl) else {
                    return PaintResult::None;
                };
                Self::paint_area(
                    response,
                    painter,
                    detect_hovered,
                    is_selected,
                    &screen_coords.collect::<Vec<_>>(),
                    style,
                )
            }
        }
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
            .map(|(shape1, shape2)| {
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
                    return shape1;
                }
                egui::Shape::Vec(Self::arrow(points[0], points[1], 4.0, stroke))
            })
            .collect()
    }

    fn dash(path: &[egui::Pos2], colour: egui::Color32, arrows: bool) -> Vec<egui::Shape> {
        let mut dashes =
            egui::Shape::dashed_line(path, egui::Stroke::new(6.0, egui::Color32::BLACK), 8.0, 8.0);
        if arrows {
            dashes = Self::add_arrows(dashes);
        }

        let mut dashes2 = egui::Shape::dashed_line(path, egui::Stroke::new(2.0, colour), 8.0, 8.0);
        if arrows {
            dashes2 = Self::add_arrows(dashes2);
        }

        dashes.extend(dashes2);
        dashes
    }
    fn hover_dash(path: &[egui::Pos2], arrows: bool) -> Vec<egui::Shape> {
        Self::dash(path, egui::Color32::WHITE, arrows)
    }

    fn select_dash(path: &[egui::Pos2], arrows: bool) -> Vec<egui::Shape> {
        Self::dash(path, egui::Color32::YELLOW, arrows)
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
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        is_selected: bool,
        nodes: &[PlaNodeScreen],
        style: &[AreaStyle],
    ) -> PaintResult {
        let mut is_hovered = !detect_hovered;

        let mut hover_coords = Vec::new();
        let mut hover_coords_is_filled = false;

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
                            hover_coords.extend(shape.flatten(TOLERANCE));
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
                            hover_coords.extend(shape.flatten(TOLERANCE));
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

            // let polygon_edge = polygon.difference(&polygon.buffer(16.0 * outline_width));

            if !is_hovered
                && let Some(hover_pos) = response.hover_pos()
                && polygon.contains(&geo::point! { x: hover_pos.x, y: hover_pos.y })
            {
                is_hovered = true;
            }

            let coords = polygon
                .exterior()
                .coords()
                .map(|a| a.to_egui_pos2())
                .collect::<Vec<_>>();
            painter.add(egui::Shape::convex_polygon(
                coords,
                colour.unwrap_or(egui::Color32::TRANSPARENT),
                egui::Stroke::new(
                    *outline_width,
                    outline.unwrap_or(egui::Color32::TRANSPARENT),
                ),
            ));
            painter.add(shapes);
        }

        PaintResult::from_conditions(is_selected, detect_hovered, is_hovered, hover_coords)
    }
    pub fn paint_line(
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        is_selected: bool,
        nodes: &[PlaNodeScreen],
        style: &[LineStyle],
    ) -> PaintResult {
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
                                            previous_coord.to_geo_coord_f32(),
                                            coord.to_geo_coord_f32(),
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
                                    .flatten(TOLERANCE)
                                    .into_iter()
                                    .map(CoordConversionExt::to_geo_coord_f32)
                                    .collect::<Vec<_>>();
                                if !hover_coords_is_filled {
                                    hover_coords.extend(approx.iter().map(|a| a.to_egui_pos2()));
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
                                    .flatten(TOLERANCE)
                                    .into_iter()
                                    .map(CoordConversionExt::to_geo_coord_f32)
                                    .collect::<Vec<_>>();
                                if !hover_coords_is_filled {
                                    hover_coords.extend(approx.iter().map(|a| a.to_egui_pos2()));
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

        PaintResult::from_conditions(is_selected, detect_hovered, is_hovered, hover_coords)
    }
    pub fn paint_point(
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        detect_hovered: bool,
        is_selected: bool,
        coord: egui::Pos2,
        style_name: &str,
        styles: &[PointStyle],
    ) -> PaintResult {
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

        let hover_coords = || {
            let dimensions = styles
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
        };
        if is_selected && detect_hovered && is_hovered {
            PaintResult::HoveredAndSelected(hover_coords())
        } else if is_selected {
            PaintResult::Selected(hover_coords())
        } else if detect_hovered && is_hovered {
            PaintResult::Hovered(hover_coords())
        } else {
            PaintResult::None
        }
    }
}
