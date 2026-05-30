use std::{
    f32::consts::FRAC_PI_2,
    num::NonZeroUsize,
    sync::{LazyLock, Mutex, MutexGuard},
};

use geo::{
    Area, BooleanOps, Buffer, Contains, CoordsIter, MapCoords, SimplifyVw, TriangulateDelaunay,
    buffer::{BufferStyle, LineJoin},
    triangulate_delaunay::{DelaunayTriangulationConfig, TriangulationResult},
};
use lru::LruCache;
use ordered_float::OrderedFloat;

use crate::{
    component_actions::paint::{TOLERANCE, utils::PartialPaintResult},
    map::MapWindow,
    project::{
        pla3::{PlaNodeScreen, PlaNodeScreenVec},
        skin::AreaStyle,
    },
    utils::coord::{CoordFrom, CoordInto},
};

type TriangulationCache =
    LruCache<geo::MultiPolygon<OrderedFloat<f32>>, TriangulationResult<Vec<[egui::Pos2; 3]>>>;

static TRIANGULATION_CACHE: LazyLock<Mutex<TriangulationCache>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(0x1_0000).unwrap())));

impl MapWindow {
    fn triangulate<'a>(
        p: &geo::MultiPolygon<f32>,
        cache: &'a mut MutexGuard<TriangulationCache>,
    ) -> &'a TriangulationResult<Vec<[egui::Pos2; 3]>> {
        let cache_key = p.map_coords(|c| geo::coord! {x: OrderedFloat(c.x), y: OrderedFloat(c.y)});

        cache.get_or_insert(cache_key, || {
            p.simplify_vw(1.0)
                .constrained_triangulation(DelaunayTriangulationConfig::default())
                .map(|a| {
                    a.iter()
                        .map(|a| a.to_array().map(CoordInto::coord_into))
                        .collect::<Vec<_>>()
                })
        })
    }

    #[tracing::instrument(skip_all)]
    #[expect(clippy::too_many_lines)]
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
                            vec![points[0].coord_into(), points[1].coord_into()]
                        }
                        egui::Shape::QuadraticBezier(shape) => shape
                            .flatten(TOLERANCE)
                            .into_iter()
                            .map(geo::Coord::<f32>::coord_from)
                            .collect(),
                        egui::Shape::CubicBezier(shape) => shape
                            .flatten(TOLERANCE)
                            .into_iter()
                            .map(geo::Coord::<f32>::coord_from)
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
                && let Some(hover_pos) = response.hover_pos().map(geo::Coord::<f32>::coord_from)
                && polygon_edge.as_ref().map_or_else(
                    || polygon.contains(&hover_pos),
                    |polygon_edge| polygon_edge.contains(&hover_pos),
                )
            {
                is_hovered = true;
            }

            let screen_boundaries = geo::Polygon::from(geo::Rect::new::<geo::Coord<f32>>(
                response.rect.max.coord_into(),
                response.rect.min.coord_into(),
            ));
            let polygon = polygon.intersection(&screen_boundaries);
            let polygon_edge =
                polygon_edge.map(|polygon_edge| polygon_edge.intersection(&screen_boundaries));

            let mut cache = TRIANGULATION_CACHE.lock().unwrap();

            if polygon_edge.is_some()
                && let Ok(fill_triangles) = Self::triangulate(&polygon, &mut cache)
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
            if let Ok(edge_triangles) = if let Some(polygon_edge) = polygon_edge {
                Self::triangulate(&polygon_edge, &mut cache)
            } else {
                Self::triangulate(&polygon, &mut cache)
            } {
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
}
