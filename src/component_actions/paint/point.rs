use crate::{
    component_actions::paint::utils::PartialPaintResult, map::MapWindow, project::skin::PointStyle,
};

impl MapWindow {
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
