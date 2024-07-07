use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

use bevy::{color::palettes::basic::YELLOW, prelude::*};
use bevy_prototype_lyon::prelude::*;
use egui_notify::ToastLevel;
use hex_color::HexColor;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    component::skin::{AreaStyle, LineStyle, PointStyle, Skin, SkinComponent},
    ui::notif::{NotifLogRwLockExt, NOTIF_LOG},
};

fn hex_to_color(hex: HexColor) -> Color {
    Color::srgba(
        f32::from(hex.r) / 255.0,
        f32::from(hex.g) / 255.0,
        f32::from(hex.b) / 255.0,
        1.0,
    )
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Component)]
pub struct PlaComponent<T: Coords> {
    pub namespace: String,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub layer: f64,
    #[serde(rename = "type")]
    pub ty: String,
    pub nodes: Vec<T>,
    #[serde(skip)]
    pub attributes: HashMap<String, String>,
}

impl<T: Coords + PartialEq> Display for PlaComponent<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.namespace, self.id)?;
        if !self.display_name.is_empty() {
            write!(f, " ({})", self.display_name)?;
        }
        Ok(())
    }
}

impl<T: Coords + PartialEq> PlaComponent<T> {
    #[must_use]
    pub fn new(ty: ComponentType) -> Self {
        Self {
            ty: format!(
                "simple{}",
                match ty {
                    ComponentType::Point => "Point",
                    ComponentType::Line => "Line",
                    ComponentType::Area => "Area",
                }
            ),
            namespace: "_misc".into(),
            ..default()
        }
    }
    #[must_use]
    pub fn get_type(&self, skin: &Skin) -> ComponentType {
        if let Some(sc) = skin.types.get(self.ty.as_str()) {
            sc.get_type()
        } else {
            let (ty, s) = if self.nodes.len() == 1 || self.nodes.iter().dedup().count() == 1 {
                (ComponentType::Point, "point")
            } else if self.nodes.first() == self.nodes.last() && !self.nodes.is_empty() {
                (ComponentType::Area, "area")
            } else {
                (ComponentType::Line, "line")
            };
            NOTIF_LOG.push(
                &format!(
                    "Unknown type {} for component {}\nAssuming it is a(n) {}",
                    self.ty, self, s
                ),
                ToastLevel::Warning,
            );
            ty
        }
    }
    #[must_use]
    pub fn front_colour<'a>(&self, skin: &'a Skin) -> Option<&'a HexColor> {
        let type_layers = skin.types.get(self.ty.as_str())?;
        match type_layers {
            SkinComponent::Point { style, .. } => style_in_max_zoom(style)?
                .iter()
                .filter_map(|style| match style {
                    PointStyle::Circle { colour, .. } | PointStyle::Square { colour, .. } => {
                        Some(colour)
                    }
                    _ => None,
                })
                .last(),
            SkinComponent::Line { style, .. } => style_in_max_zoom(style)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Fore { colour, .. } => Some(colour),
                    _ => None,
                })
                .last(),
            SkinComponent::Area { style, .. } => style_in_max_zoom(style)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { colour, .. } => colour.into(),
                    _ => None,
                })
                .last(),
        }
    }
    #[must_use]
    pub fn back_colour<'a>(&self, skin: &'a Skin) -> Option<&'a HexColor> {
        let type_layers = skin.types.get(self.ty.as_str())?;
        match type_layers {
            SkinComponent::Point { .. } => None,
            SkinComponent::Line { style, .. } => style_in_max_zoom(style)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Back { colour, .. } => Some(colour),
                    _ => None,
                })
                .last(),
            SkinComponent::Area { style, .. } => style_in_max_zoom(style)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { outline, .. } => outline.into(),
                    _ => None,
                })
                .last(),
        }
    }
    #[must_use]
    pub fn weight(&self, skin: &Skin) -> Option<u32> {
        let type_layers = skin.types.get(self.ty.as_str())?;
        match type_layers {
            SkinComponent::Point { .. } => None,
            SkinComponent::Line { style, .. } => style_in_max_zoom(style)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Fore { width, .. } => Some(width / 4),
                    _ => None,
                })
                .last(),
            SkinComponent::Area { .. } => Some(2),
        }
    }
}

impl PlaComponent<MCCoords> {
    #[must_use]
    pub fn to_editor_coords(&self) -> PlaComponent<EditorCoords> {
        PlaComponent {
            namespace: self.namespace.to_owned(),
            id: self.id.to_owned(),
            display_name: self.display_name.to_owned(),
            description: self.description.to_owned(),
            tags: self.tags.to_owned(),
            layer: self.layer,
            ty: self.ty.to_owned(),
            nodes: self.nodes.iter().map(|a| (*a).into()).collect(),
            attributes: self.attributes.to_owned(),
        }
    }
}

impl PlaComponent<EditorCoords> {
    #[must_use]
    pub fn to_mc_coords(&self) -> PlaComponent<MCCoords> {
        PlaComponent {
            namespace: self.namespace.to_owned(),
            id: self.id.to_owned(),
            display_name: self.display_name.to_owned(),
            description: self.description.to_owned(),
            tags: self.tags.to_owned(),
            layer: self.layer,
            ty: self.ty.to_owned(),
            nodes: self.nodes.iter().map(|a| (*a).into()).collect(),
            attributes: self.attributes.to_owned(),
        }
    }

    #[must_use]
    pub fn get_shape(&self, skin: &Skin) -> ShapeBundle {
        if self.get_type(skin) == ComponentType::Point {
            return ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: Vec2::splat(2.0),
                    origin: RectangleOrigin::Center,
                }),
                spatial: SpatialBundle::from_transform(Transform::from_xyz(
                    self.nodes[0].0.x as f32,
                    self.nodes[0].0.y as f32,
                    10.0,
                )),
                ..default()
            };
        }
        let path = GeometryBuilder::build_as(&{
            let mut pb = PathBuilder::new();
            for coord in &self.nodes {
                pb.line_to(coord.0.as_vec2());
            }
            if self.get_type(skin) == ComponentType::Area {
                if let Some(coord) = self.nodes.first() {
                    pb.line_to(coord.0.as_vec2());
                }
            }
            pb.build()
        });
        let transform = Transform::from_xyz(0.0, 0.0, {
            let order = skin
                .order
                .iter()
                .enumerate()
                .find(|(_, a)| **a == self.ty)
                .map_or(0, |a| a.0);
            (order as f32).mul_add(f32::EPSILON, self.layer as f32 + 20.0)
        });
        ShapeBundle {
            path,
            spatial: SpatialBundle::from_transform(transform),
            ..default()
        }
    }

    #[must_use]
    pub fn get_fill(&self, skin: &Skin) -> Fill {
        if self.get_type(skin) == ComponentType::Point {
            return Fill::color(if let Some(hex) = self.front_colour(skin) {
                hex_to_color(*hex)
            } else {
                Color::WHITE
            });
        }
        if self.get_type(skin) == ComponentType::Area {
            Fill::color(if let Some(hex) = self.front_colour(skin) {
                hex_to_color(*hex).with_alpha(0.25)
            } else {
                Color::NONE
            })
        } else {
            Fill::color(Color::NONE)
        }
    }

    #[must_use]
    pub fn get_stroke(&self, skin: &Skin) -> Stroke {
        if self.get_type(skin) == ComponentType::Point {
            return Stroke::color(Color::NONE);
        }
        let options = StrokeOptions::default()
            .with_start_cap(LineCap::Round)
            .with_end_cap(LineCap::Round)
            .with_line_join(LineJoin::Round)
            .with_line_width(self.weight(skin).unwrap_or(2) as f32);
        if self.get_type(skin) == ComponentType::Area {
            Stroke {
                color: if let Some(hex) = self.back_colour(skin) {
                    hex_to_color(*hex)
                } else {
                    Color::NONE
                },
                options,
            }
        } else {
            Stroke {
                color: if let Some(hex) = self.front_colour(skin) {
                    hex_to_color(*hex)
                } else {
                    Color::WHITE
                },
                options,
            }
        }
    }
}

pub trait Select {
    fn select(&mut self, ty: ComponentType) -> &mut Self;
}
impl Select for Fill {
    fn select(&mut self, ty: ComponentType) -> &mut Self {
        self.color = match ty {
            ComponentType::Point => YELLOW.into(),
            ComponentType::Line => Color::NONE,
            ComponentType::Area => YELLOW.with_alpha(0.25).into(),
        };
        self
    }
}
impl Select for Stroke {
    fn select(&mut self, ty: ComponentType) -> &mut Self {
        self.color = match ty {
            ComponentType::Point => Color::NONE,
            ComponentType::Line | ComponentType::Area => YELLOW.into(),
        };
        self
    }
}

fn style_in_max_zoom<T>(style: &HashMap<String, Vec<T>>) -> Option<&Vec<T>> {
    Some(
        style
            .iter()
            .map(|(zl, data)| (zl.split(", ").next().unwrap().parse::<u8>().unwrap(), data))
            .find(|(min, _)| *min == 0)?
            .1,
    )
}

pub trait Coords: Debug + Default + Copy + Clone {}

#[derive(Component, Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct MCCoords(pub Vec2);

#[derive(Component, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCoords(pub IVec2);

impl From<EditorCoords> for MCCoords {
    fn from(c: EditorCoords) -> Self {
        Self(Vec2::new(c.0.x as f32, -c.0.y as f32))
    }
}

impl From<MCCoords> for EditorCoords {
    fn from(c: MCCoords) -> Self {
        Self(IVec2::new(c.0.x as i32, -c.0.y as i32))
    }
}

impl From<IVec2> for EditorCoords {
    fn from(c: IVec2) -> Self {
        Self(c)
    }
}

impl Coords for MCCoords {}

impl Coords for EditorCoords {}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ComponentType {
    #[serde(rename = "point")]
    Point,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "area")]
    Area,
}
