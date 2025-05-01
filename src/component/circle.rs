use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::tile::zoom::Zoom;

#[must_use]
pub fn circle(zoom: &Zoom, center: Vec2, multiplier: f32, color: Color) -> Shape {
    ShapeBuilder::with(&shapes::Circle {
        radius: 768.0 / zoom.0.exp2() * multiplier,
        center,
    })
    .fill(Fill::color(Color::WHITE))
    .stroke(Stroke::new(color, 512.0 / zoom.0.exp2() * multiplier))
    .build()
}
