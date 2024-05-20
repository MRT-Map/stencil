use bevy::prelude::*;
use bevy_prototype_lyon::{
    draw::{Fill, Stroke},
    entity::ShapeBundle,
    geometry::GeometryBuilder,
    shapes::Circle,
};

use crate::{
    component::{
        pla2::{EditorCoords, PlaComponent},
        skin::Skin,
    },
    tile::zoom::Zoom,
};

#[must_use]
pub fn circle(
    data: &PlaComponent<EditorCoords>,
    skin: &Skin,
    zoom: &Zoom,
    center: Vec2,
    multiplier: f32,
    color: Color,
) -> (ShapeBundle, Fill, Stroke) {
    let weight = data.weight(skin).unwrap_or(2) as f32;
    (
        ShapeBundle {
            path: GeometryBuilder::build_as(&Circle {
                radius: weight * 512.0 / zoom.0.exp2() * multiplier,
                center,
            }),
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 100.0)),
            ..default()
        },
        Fill::color(Color::WHITE),
        Stroke::new(color, weight * 512.0 / zoom.0.exp2() * multiplier),
    )
}
