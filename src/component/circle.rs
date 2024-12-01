use bevy::prelude::*;
use bevy_prototype_lyon::{
    draw::{Fill, Stroke},
    entity::ShapeBundle,
    geometry::GeometryBuilder,
    shapes::Circle,
};

use crate::tile::zoom::Zoom;

#[must_use]
pub fn circle(
    zoom: &Zoom,
    center: Vec2,
    multiplier: f32,
    color: Color,
) -> (ShapeBundle, Fill, Stroke) {
    (
        ShapeBundle {
            path: GeometryBuilder::build_as(&Circle {
                radius: 1024.0 / zoom.0.exp2() * multiplier,
                center,
            }),
            transform: Transform::from_xyz(0.0, 0.0, 100.0),
            ..default()
        },
        Fill::color(Color::WHITE),
        Stroke::new(color, 1024.0 / zoom.0.exp2() * multiplier),
    )
}
