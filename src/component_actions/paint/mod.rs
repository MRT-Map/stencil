mod area;
mod line;
mod point;
mod utils;

use std::borrow::Cow;

use tracing::debug;
use utils::PaintResult;

use crate::{
    App,
    map::MapWindow,
    project::{
        pla3::{PlaComponent, PlaNodeScreen, ToScreenExt},
        skin::SkinType,
    },
    utils::coord::{CoordFrom, Nnf32},
};

pub const TOLERANCE: Option<f32> = Some(1.0);

impl MapWindow {
    #[tracing::instrument(skip_all)]
    pub fn paint_components(app: &mut App, response: &egui::Response, painter: &egui::Painter) {
        let mut hovered_component = None;
        let mut selected_shapes = Vec::new();
        let mut all_shapes: Vec<egui::Shape> = Vec::new();
        for component in app.project.components.iter().rev() {
            let is_selected = app.ui.map.is_selected(&component.full_id);
            let Some(PaintResult {
                is_hovered: is_hovering,
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
        for shape in all_shapes {
            painter.add(shape);
        }
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
    #[tracing::instrument(skip_all, fields(%id = component.full_id))]
    pub fn paint_component<'a>(
        app: &App,
        response: &egui::Response,
        detect_hovered: bool,
        is_selected: bool,
        component: &'a PlaComponent,
    ) -> Option<PaintResult<'a>> {
        let bounding_box = component
            .nodes
            .clone()
            .map(egui::Pos2::coord_from)
            .bounding_box()
            .map(geo::Rect::<Nnf32>::coord_from)?;
        let world_boundaries = app.map_world_boundaries(response.rect);
        if world_boundaries.max().x < bounding_box.min().x
            || bounding_box.max().x < world_boundaries.min().x
            || world_boundaries.max().y < bounding_box.min().y
            || bounding_box.max().y < world_boundaries.min().y
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
            is_hovered: result.is_hovered,
            shapes: result.shapes,
            screen_coords,
            point_style,
        })
    }
}
