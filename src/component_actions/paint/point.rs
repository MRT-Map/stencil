use crate::{
    component_actions::paint::utils::PartialPaintResult, map::MapWindow, project::skin::PointStyle,
};

impl MapWindow {
    fn paint_point_style(
        response: &egui::Response,
        detect_hovered: bool,
        coord: egui::Pos2,
        style_name: &str,
        style: &PointStyle,
    ) -> Option<PartialPaintResult> {
        let shape = match style {
            PointStyle::Image {
                image,
                size,
                offset,
                extension,
                ..
            } => Self::image_shape_from_bytes(
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
            )?,
            PointStyle::Square {
                colour,
                border_radius,
                size,
                ..
            } => egui::Shape::rect_filled(
                egui::Rect::from_center_size(coord, egui::Vec2::splat(*size * 4.0)),
                *border_radius,
                colour.unwrap_or_default(),
            ),
            PointStyle::Text { .. } => return None,
        };

        let is_hovered = detect_hovered
            && response
                .hover_pos()
                .is_some_and(|hover_pos| shape.visual_bounding_rect().contains(hover_pos));
        Some(PartialPaintResult {
            shapes: vec![shape],
            is_hovered,
        })
    }
    #[tracing::instrument(skip_all)]
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
            let Some(PartialPaintResult {
                is_hovered: style_is_hovered,
                shapes: style_shapes,
            }) = Self::paint_point_style(response, !is_hovered, coord, style_name, style)
            else {
                continue;
            };
            is_hovered |= style_is_hovered;
            shapes.extend(style_shapes);
        }

        PartialPaintResult {
            is_hovered: detect_hovered && is_hovered,
            shapes,
        }
    }
}
