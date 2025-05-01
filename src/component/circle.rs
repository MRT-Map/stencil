use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::ui::map::zoom::Zoom;

#[must_use]
pub fn make_circle(zoom: &Zoom, center: Vec2, multiplier: f32, color: Color) -> impl Bundle {
    (
        ShapeBuilder::with(&shapes::Circle {
            radius: 768.0 / zoom.0.exp2() * multiplier,
            center,
        })
        .fill(Fill::color(Color::WHITE))
        .stroke(Stroke::new(color, 512.0 / zoom.0.exp2() * multiplier))
        .build(),
        Transform::from_xyz(0.0, 0.0, 100.0),
    )
}
