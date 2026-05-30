use geo::Distance;

use crate::{
    component_actions::paint::{TOLERANCE, utils::PartialPaintResult},
    map::MapWindow,
    project::{
        pla3::{PlaNodeScreen, PlaNodeScreenVec},
        skin::LineStyle,
    },
    utils::coord::CoordInto,
};

impl MapWindow {
    fn hovering<L>(is_hovered: &mut bool, response: &egui::Response, width: f32, line: &L)
    where
        geo::Euclidean: for<'a> Distance<f32, &'a L, &'a geo::Point<f32>>,
    {
        if !*is_hovered
            && let Some(hover_pos) = response.hover_pos()
            && geo::Euclidean.distance(line, &geo::point! { x: hover_pos.x, y: hover_pos.y })
                < width / 2.0 * 1.5
        {
            *is_hovered = true;
        }
    }

    #[tracing::instrument(skip_all)]
    #[expect(clippy::too_many_lines)]
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
                                    Self::hovering(
                                        &mut is_hovered,
                                        response,
                                        width,
                                        &geo::Line::new::<geo::Coord<f32>>(
                                            previous_coord.coord_into(),
                                            coord.coord_into(),
                                        ),
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
                                    .map(CoordInto::coord_into)
                                    .collect::<Vec<_>>();
                                Self::hovering(
                                    &mut is_hovered,
                                    response,
                                    width,
                                    &geo::LineString::new(approx),
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
                                    .map(CoordInto::coord_into)
                                    .collect::<Vec<_>>();
                                Self::hovering(
                                    &mut is_hovered,
                                    response,
                                    width,
                                    &geo::LineString::new(approx),
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
}
