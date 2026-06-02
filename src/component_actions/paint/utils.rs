use std::borrow::Cow;

use itertools::Itertools;
use tracing::error;

use crate::{
    component_actions::paint::TOLERANCE,
    map::MapWindow,
    project::{
        pla3::{PlaNodeScreen, PlaNodeScreenVec},
        skin::PointStyle,
    },
};

impl MapWindow {
    pub(super) fn outline(
        point_style: Option<&[PointStyle]>,
        nodes: &PlaNodeScreenVec,
    ) -> Vec<egui::Pos2> {
        let Some(style) = point_style else {
            return nodes.outline(TOLERANCE);
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

    pub(super) fn image_shape_from_bytes(
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
        .inspect_err(|e| error!("{e:#}"))
        .ok()
        .and_then(|a| a.texture_id())?;

        Some(egui::Shape::image(
            texture_id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        ))
    }
}

#[must_use]
pub struct PaintResult<'a> {
    pub shapes: Vec<egui::Shape>,
    pub is_hovered: bool,
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
    pub is_hovered: bool,
}

impl PartialPaintResult {
    pub fn paint(self, painter: &egui::Painter) {
        painter.add(self.shapes);
    }
}
