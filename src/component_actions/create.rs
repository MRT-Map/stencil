use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock},
};

use geo::Vector2DOps;
use itertools::{Either, Itertools};
use ordered_float::NotNan;
use pla3::FullId;
use tracing::info;

use crate::{
    App,
    component_actions::event::ComponentEv,
    coord::CoordInto,
    map::MapWindow,
    pointer::ResponsePointerExt,
    project::pla3::{PlaComponent, PlaNodeWorld, ToScreenExt},
};

static ANGLE_VECTORS: LazyLock<[geo::Coord<f32>; 40]> = LazyLock::new(|| {
    let vec: [geo::Coord<f32>; 20] = [
        (4.0, 0.0),
        (4.0, 1.0),
        (3.0, 1.0),
        (2.0, 1.0),
        (1.5, 1.0),
        (1.0, 1.0),
        (1.0, 1.5),
        (1.0, 2.0),
        (1.0, 3.0),
        (1.0, 4.0),
        (0.0, 4.0),
        (-1.0, 4.0),
        (-1.0, 3.0),
        (-1.0, 2.0),
        (-1.0, 1.5),
        (-1.0, 1.0),
        (-1.5, 1.0),
        (-2.0, 1.0),
        (-3.0, 1.0),
        (-4.0, 1.0),
    ]
    .map(|(x, y)| geo::coord! { x: x, y: y }.try_normalize().unwrap());
    vec.into_iter()
        .chain(vec.into_iter().map(|a| -a))
        .collect_array()
        .unwrap()
});

impl MapWindow {
    #[tracing::instrument(skip_all)]
    pub fn create_point(
        app: &mut App,
        ctx: &egui::Context,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        if app.project.new_component_ns.is_empty() {
            return;
        }
        let (Some(cursor_world_pos), Some(skin)) =
            (app.ui.map.cursor_world_pos, app.project.skin())
        else {
            return;
        };
        let Some(ty) = app
            .ui
            .map
            .created_point_type
            .as_ref()
            .or_else(|| skin.get_type("simplePoint"))
        else {
            return;
        };
        let Some(style) = ty.point_style_in_zoom_level(app.map_zoom_level()) else {
            return;
        };

        let world_coord: geo::Coord<i32> = cursor_world_pos.coord_into();
        let screen_coord =
            app.map_world_to_screen(response.rect.center(), world_coord.coord_into());
        Self::paint_point(response, false, screen_coord, ty.name(), style).paint(painter);

        if !response.clicked_by2(egui::PointerButton::Primary) {
            return;
        }
        let component = PlaComponent {
            full_id: FullId::new(
                app.project.new_component_ns.clone(),
                app.project
                    .components
                    .get_new_id(&app.project.new_component_ns),
            ),
            ty: Arc::clone(ty),
            display_name: String::new(),
            layer: NotNan::<f32>::default(),
            nodes: vec![PlaNodeWorld::Line {
                coord: world_coord,
                label: None,
            }]
            .into(),
            misc: BTreeMap::default(),
        };
        app.status_on_create("point", &component);
        app.run_event(ComponentEv::Create(vec![component]), ctx);
    }
    #[inline]
    #[tracing::instrument(skip_all)]
    pub fn create_line(
        app: &mut App,
        ctx: &egui::Context,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        Self::create_line_or_area::<true>(app, ctx, response, painter);
    }
    #[inline]
    #[tracing::instrument(skip_all)]
    pub fn create_area(
        app: &mut App,
        ctx: &egui::Context,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        Self::create_line_or_area::<false>(app, ctx, response, painter);
    }
    pub fn create_line_or_area<const IS_LINE: bool>(
        app: &mut App,
        ctx: &egui::Context,
        response: &egui::Response,
        painter: &egui::Painter,
    ) {
        if app.project.new_component_ns.is_empty() {
            return;
        }
        let (Some(cursor_world_pos), Some(skin)) =
            (app.ui.map.cursor_world_pos, app.project.skin())
        else {
            return;
        };
        let (ty, style) = if IS_LINE {
            let Some(ty) = app
                .ui
                .map
                .created_line_type
                .as_ref()
                .or_else(|| skin.get_type("simpleLine"))
            else {
                return;
            };

            let Some(style) = ty.line_style_in_zoom_level(app.map_zoom_level()) else {
                return;
            };
            (Either::Left(ty), Either::Left(style))
        } else {
            let Some(ty) = app
                .ui
                .map
                .created_area_type
                .as_ref()
                .or_else(|| skin.get_type("simpleArea"))
            else {
                return;
            };

            let Some(style) = ty.area_style_in_zoom_level(app.map_zoom_level()) else {
                return;
            };
            (Either::Right(ty), Either::Right(style))
        };

        let mut world_coord: geo::Coord<i32> = cursor_world_pos.coord_into();

        if ctx.input(|a| a.modifiers.command)
            && let Some(prev_coord) = match app.ui.map.created_nodes.last() {
                Some(PlaNodeWorld::Line { .. }) if app.ui.map.created_nodes.len() > 1 => {
                    app.ui.map.created_nodes.second_last().map(|a| a.coord())
                }
                Some(PlaNodeWorld::QuadraticBezier { ctrl, .. }) => Some(*ctrl),
                Some(PlaNodeWorld::CubicBezier { ctrl2, .. }) => Some(*ctrl2),
                _ => None,
            }
            && world_coord != prev_coord
        {
            let angle_vec: geo::Coord<f32> = (world_coord - prev_coord).coord_into();
            let (closest_angle_vec, _) = ANGLE_VECTORS
                .into_iter()
                .map(|v| (v, v.dot_product(angle_vec.try_normalize().unwrap())))
                .sorted_by(|(_, k1), (_, k2)| k1.total_cmp(k2))
                .next()
                .unwrap();
            // adapted from https://docs.rs/glam/latest/src/glam/f32/vec2.rs.html#618-622
            let world_coord_f32 = closest_angle_vec * angle_vec.dot_product(closest_angle_vec)
                / closest_angle_vec.dot_product(closest_angle_vec);
            world_coord = prev_coord + world_coord_f32.coord_into();
        }

        match app.ui.map.created_nodes.last_mut() {
            None => app.ui.map.created_nodes.push(PlaNodeWorld::Line {
                coord: world_coord,
                label: None,
            }),
            Some(node) => *node.coord_mut() = world_coord,
        }

        let screen_nodes = app
            .ui
            .map
            .created_nodes
            .to_screen(app, response.rect.center());
        match style {
            Either::Left(style) => {
                Self::paint_line(response, false, &screen_nodes, style).paint(painter);
            }
            Either::Right(style) => {
                Self::paint_area(response, false, &screen_nodes, style).paint(painter);
            }
        }

        if let Some(curve_vec) = match app.ui.map.created_nodes.last_chunk::<2>() {
            Some(
                [
                    second_last,
                    PlaNodeWorld::QuadraticBezier { ctrl, coord, .. },
                ],
            ) => Some(vec![second_last.coord(), *ctrl, *coord]),
            Some(
                [
                    second_last,
                    PlaNodeWorld::CubicBezier {
                        ctrl1,
                        ctrl2,
                        coord,
                        ..
                    },
                ],
            ) => Some(vec![second_last.coord(), *ctrl1, *ctrl2, *coord]),
            Some(
                [
                    PlaNodeWorld::Line { coord: coord1, .. },
                    PlaNodeWorld::Line { coord: coord2, .. },
                ],
            ) => {
                (!IS_LINE && app.ui.map.created_nodes.len() == 2).then_some(vec![*coord1, *coord2])
            }
            _ => None,
        } {
            let curve_vec = curve_vec
                .iter()
                .map(|a| app.map_world_to_screen(response.rect.center(), a.coord_into()))
                .collect::<Vec<_>>();
            painter.add(Self::white_dash(&curve_vec, false));
        }

        if response.clicked_by2(egui::PointerButton::Secondary) {
            let last_node = app.ui.map.created_nodes.last_mut().unwrap();
            info!(?last_node, "Undoing last control point / node");
            match *last_node {
                PlaNodeWorld::Line { .. } => {
                    app.ui.map.created_nodes.pop();
                }
                PlaNodeWorld::QuadraticBezier { coord, label, .. } => {
                    *last_node = PlaNodeWorld::Line { coord, label }
                }
                PlaNodeWorld::CubicBezier {
                    ctrl1,
                    coord,
                    label,
                    ..
                } => {
                    *last_node = PlaNodeWorld::QuadraticBezier {
                        ctrl: ctrl1,
                        coord,
                        label,
                    }
                }
            }
        } else if response.clicked_by2(egui::PointerButton::Primary) {
            if ctx.input(|a| a.modifiers.shift) && app.ui.map.created_nodes.len() > 1 {
                let last_node = app.ui.map.created_nodes.last_mut().unwrap();
                match *last_node {
                    PlaNodeWorld::Line { coord, label } => {
                        *last_node = PlaNodeWorld::QuadraticBezier {
                            ctrl: coord,
                            coord,
                            label,
                        }
                    }
                    PlaNodeWorld::QuadraticBezier { ctrl, coord, label } => {
                        *last_node = PlaNodeWorld::CubicBezier {
                            ctrl1: ctrl,
                            ctrl2: coord,
                            coord,
                            label,
                        }
                    }
                    PlaNodeWorld::CubicBezier { .. } => {}
                }
                info!(?last_node, "Adding control point");
            } else if app
                .ui
                .map
                .created_nodes
                .last_chunk::<2>()
                .is_none_or(|[sl, l]| sl.coord() != l.coord())
            {
                app.ui.map.created_nodes.push(PlaNodeWorld::Line {
                    coord: world_coord,
                    label: None,
                });
                info!(?world_coord, "Adding node");
            }
        }
        if response.double_clicked_by2(egui::PointerButton::Primary)
            || response.double_clicked_by2(egui::PointerButton::Middle)
        {
            app.ui.map.created_nodes.pop();
            if app.ui.map.created_nodes.len() >= (if IS_LINE { 2 } else { 3 }) {
                if !IS_LINE
                    && app.ui.map.created_nodes.first().unwrap().coord()
                        != app.ui.map.created_nodes.last().unwrap().coord()
                {
                    let coord = app.ui.map.created_nodes.first().unwrap().coord();
                    app.ui
                        .map
                        .created_nodes
                        .push(PlaNodeWorld::Line { coord, label: None });
                }
                let component = PlaComponent {
                    full_id: FullId::new(
                        app.project.new_component_ns.clone(),
                        app.project
                            .components
                            .get_new_id(&app.project.new_component_ns),
                    ),
                    ty: Arc::clone(ty.into_inner()),
                    display_name: String::new(),
                    layer: NotNan::<f32>::default(),
                    nodes: app.ui.map.created_nodes.drain(..).collect(),
                    misc: BTreeMap::default(),
                };
                app.status_on_create(if IS_LINE { "line" } else { "area" }, &component);
                app.run_event(ComponentEv::Create(vec![component]), ctx);
            } else {
                app.ui.map.created_nodes.clear();
                info!(
                    "No new {} created due to too few points",
                    if IS_LINE { "line" } else { "area" }
                );
            }
        }
    }
}
