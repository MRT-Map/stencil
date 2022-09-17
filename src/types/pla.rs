use std::collections::HashMap;
use std::fmt::Debug;

use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use hex_color::HexColor;
use serde::{Deserialize, Serialize};

use crate::types::{ComponentType, skin::Skin};
use crate::types::skin::{AreaStyle, LineStyle, PointStyle, SkinComponent};

fn hex_to_color(hex: &HexColor) -> Color {
    Color::Rgba {
        red: hex.r as f32 / 255.0,
        green: hex.g as f32 / 255.0,
        blue: hex.b as f32 / 255.0,
        alpha: 1.0,
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Component)]
pub struct PlaComponent<T: Coords> {
    pub namespace: String,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub layer: f64,
    #[serde(rename = "type")]
    pub type_: String,
    pub nodes: Vec<T>,
    pub attributes: HashMap<String, String>,
}

#[allow(dead_code)]
impl<T: Coords> PlaComponent<T> {
    pub fn new(type_: ComponentType) -> Self {
        Self {
            type_: format!(
                "simple{}",
                match type_ {
                    ComponentType::Point => "Point",
                    ComponentType::Line => "Line",
                    ComponentType::Area => "Area",
                }
            ),
            ..default()
        }
    }
    pub fn get_type(&self, skin: &Skin) -> Option<ComponentType> {
        Some(skin.types.get(self.type_.as_str())?.get_type())
    }
    pub fn front_colour<'a>(&self, skin: &'a Skin) -> Option<&'a HexColor> {
        let type_layers = skin.types.get(self.type_.as_str())?;
        match type_layers {
            SkinComponent::Point { style, .. } => {
                style_in_max_zoom(style)?
                    .iter()
                    .filter_map(|style| match style {
                        PointStyle::Circle { colour, .. }
                        | PointStyle::Square { colour, .. } => Some(colour),
                        _ => None
                    })
                    .last()
            }
            SkinComponent::Line { style, .. } => {
                style_in_max_zoom(style)?
                    .iter()
                    .filter_map(|style| match style {
                        LineStyle::Fore { colour, .. } => Some(colour),
                        _ => None
                    }).last()
            }
            SkinComponent::Area { style, .. } => {
                style_in_max_zoom(style)?
                    .iter()
                    .filter_map(|style| match style {
                        AreaStyle::Fill { colour, .. } => colour.into(),
                        _ => None
                    }).last()
            }
        }
    }
    pub fn back_colour<'a>(&self, skin: &'a Skin) -> Option<&'a HexColor> {
        let type_layers = skin.types.get(self.type_.as_str())?;
        match type_layers {
            SkinComponent::Point { .. } => None,
            SkinComponent::Line { style, .. } => {
                style_in_max_zoom(style)?
                    .iter()
                    .filter_map(|style| match style {
                        LineStyle::Back { colour, .. } => Some(colour),
                        _ => None
                    }).last()
            }
            SkinComponent::Area { style, .. } => {
                style_in_max_zoom(style)?
                    .iter()
                    .filter_map(|style| match style {
                        AreaStyle::Fill { outline, .. } => outline.into(),
                        _ => None
                    }).last()
            }
        }
    }
    pub fn weight(&self, skin: &Skin) -> Option<u32> {
        let type_layers = skin.types.get(self.type_.as_str())?;
        match type_layers {
            SkinComponent::Point { .. } => None,
            SkinComponent::Line { style, .. } => {
                style_in_max_zoom(style)?
                    .iter()
                    .filter_map(|style| match style {
                        LineStyle::Fore { width, .. } => Some(width / 2),
                        _ => None
                    }).last()
            }
            SkinComponent::Area { .. } => Some(4)
        }
    }
}

impl PlaComponent<EditorCoords> {
    pub fn get_shape(&self, skin: &Skin, selected: bool) -> ShapeBundle {
        if self.get_type(skin) == Some(ComponentType::Point) {
            GeometryBuilder::build_as(
                &shapes::Rectangle {
                    extents: Vec2::new(5.0, 5.0),
                    origin: RectangleOrigin::Center,
                },
                DrawMode::Fill(FillMode::color(if selected {
                    Color::YELLOW
                } else if let Some(hex) = self.front_colour(skin) {
                    hex_to_color(hex)
                } else {
                    Color::WHITE
                })),
                Transform::from_xyz(self.nodes[0].0.x as f32, self.nodes[0].0.y as f32, 10.0),
            )
        } else {
            let options = StrokeOptions::default()
                .with_start_cap(LineCap::Round)
                .with_end_cap(LineCap::Round)
                .with_line_join(LineJoin::Round)
                .with_line_width(self.weight(skin).unwrap_or(2) as f32);
            GeometryBuilder::build_as(
                &{
                    let mut pb = PathBuilder::new();
                    for coord in &self.nodes {
                        pb.line_to(coord.0.as_vec2());
                    }
                    pb.build()
                },
                if self.get_type(skin) == Some(ComponentType::Area) {
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(if selected {
                            *Color::YELLOW.clone().set_a(0.5)
                        } else if let Some(hex) = self.front_colour(skin) {
                            *hex_to_color(hex).set_a(0.5)
                        } else {
                            Color::NONE
                        }),
                        outline_mode: StrokeMode {
                            color: if selected {
                                Color::YELLOW
                            } else if let Some(hex) = self.back_colour(skin) {
                                hex_to_color(hex)
                            } else {
                                Color::NONE
                            },
                            options,
                        },
                    }
                } else {
                    DrawMode::Stroke(StrokeMode {
                        color: if selected {
                            Color::YELLOW
                        } else if let Some(hex) = self.front_colour(skin) {
                            hex_to_color(hex)
                        } else {
                            Color::WHITE
                        },
                        options,
                    }, )
                },
                Transform::from_xyz(0.0, 0.0, 10.0),
            )
        }
    }
}

fn style_in_max_zoom<T>(style: &HashMap<String, Vec<T>>) -> Option<&Vec<T>> {
    Some(style.iter()
        .map(|(zl, data)| (
            zl.split(", ").next().unwrap().parse::<u8>().unwrap(), data)
        )
        .find(|(min, _)|
            *min == 0)?.1)
}

pub trait Coords: Debug + Default + Copy + Clone {}

#[derive(Component, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct MCCoords(pub IVec2);

#[derive(Component, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct EditorCoords(pub IVec2);

impl From<EditorCoords> for MCCoords {
    fn from(c: EditorCoords) -> Self {
        Self(IVec2::new(c.0.x, -c.0.y))
    }
}

impl From<MCCoords> for EditorCoords {
    fn from(c: MCCoords) -> Self {
        Self(IVec2::new(c.0.x, -c.0.y))
    }
}

impl From<IVec2> for EditorCoords {
    fn from(c: IVec2) -> Self {
        Self(c)
    }
}

impl Coords for MCCoords {}

impl Coords for EditorCoords {}

