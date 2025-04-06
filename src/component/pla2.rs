use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

use bevy::{
    color::palettes::basic::{OLIVE, YELLOW},
    prelude::*,
};
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
        skin.get_type(self.ty.as_str()).map_or_else(
            || {
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
            },
            SkinComponent::get_type,
        )
    }
    #[must_use]
    pub fn front_colour<'a>(&self, skin: &'a Skin) -> Option<&'a HexColor> {
        let type_layers = skin.get_type(self.ty.as_str())?;
        match type_layers {
            SkinComponent::Point { styles, .. } => style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    PointStyle::Square { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
            SkinComponent::Line { styles, .. } => style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Fore { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
            SkinComponent::Area { styles, .. } => style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
        }
    }
    #[must_use]
    pub fn back_colour<'a>(&self, skin: &'a Skin) -> Option<&'a HexColor> {
        let type_layers = skin.get_type(self.ty.as_str())?;
        match type_layers {
            SkinComponent::Point { .. } => None,
            SkinComponent::Line { styles, .. } => style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Back { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
            SkinComponent::Area { styles, .. } => style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { outline, .. } => outline.into(),
                    _ => None,
                })
                .next_back(),
        }
    }
    #[must_use]
    pub fn weight(&self, skin: &Skin) -> Option<f32> {
        let type_layers = skin.get_type(self.ty.as_str())?;
        match type_layers {
            SkinComponent::Point { .. } => None,
            SkinComponent::Line { styles, .. } => style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Fore { width, .. } => Some(*width),
                    _ => None,
                })
                .next_back(),
            SkinComponent::Area { styles, .. } => style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { outline_width, .. } => Some(outline_width * 5.0),
                    _ => None,
                })
                .next_back(),
        }
    }
}

impl PlaComponent<MCCoords> {
    #[must_use]
    pub fn to_editor_coords(&self) -> PlaComponent<EditorCoords> {
        PlaComponent {
            namespace: self.namespace.clone(),
            id: self.id.clone(),
            display_name: self.display_name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            layer: self.layer,
            ty: self.ty.clone(),
            nodes: self.nodes.iter().map(|a| (*a).into()).collect(),
            attributes: self.attributes.clone(),
        }
    }
}

impl PlaComponent<EditorCoords> {
    #[must_use]
    pub fn to_mc_coords(&self) -> PlaComponent<MCCoords> {
        PlaComponent {
            namespace: self.namespace.clone(),
            id: self.id.clone(),
            display_name: self.display_name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            layer: self.layer,
            ty: self.ty.clone(),
            nodes: self.nodes.iter().map(|a| (*a).into()).collect(),
            attributes: self.attributes.clone(),
        }
    }

    #[must_use]
    pub fn get_shape(&self, skin: &Skin) -> ShapeBundle {
        if self.get_type(skin) == ComponentType::Point {
            return ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: Vec2::splat(2.0),
                    origin: RectangleOrigin::Center,
                    ..default()
                }),
                transform: Transform::from_xyz(
                    self.nodes[0].0.x as f32,
                    self.nodes[0].0.y as f32,
                    10.0,
                ),
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
            let order = skin.get_order(&self.ty).unwrap_or(0);
            (order as f32).mul_add(0.001, self.layer as f32 + 20.0)
        });
        ShapeBundle {
            path,
            transform,
            ..default()
        }
    }

    #[must_use]
    pub fn get_fill(&self, skin: &Skin) -> Fill {
        if self.get_type(skin) == ComponentType::Point {
            return Fill::color(
                self.front_colour(skin)
                    .map_or(Color::WHITE, |hex| hex_to_color(*hex)),
            );
        }
        if self.get_type(skin) == ComponentType::Area {
            Fill::color(
                self.front_colour(skin)
                    .map_or(Color::NONE, |hex| hex_to_color(*hex).with_alpha(0.25)),
            )
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
            .with_line_width(self.weight(skin).unwrap_or(2.0));
        if self.get_type(skin) == ComponentType::Area {
            Stroke {
                color: self
                    .back_colour(skin)
                    .map_or(Color::NONE, |hex| hex_to_color(*hex)),
                options,
            }
        } else {
            Stroke {
                color: self
                    .front_colour(skin)
                    .map_or(Color::WHITE, |hex| hex_to_color(*hex)),
                options,
            }
        }
    }
}

pub trait HighlightExt {
    fn select(&mut self, ty: ComponentType) -> &mut Self;
    fn hover(&mut self, ty: ComponentType) -> &mut Self;
}
impl HighlightExt for Fill {
    fn select(&mut self, ty: ComponentType) -> &mut Self {
        self.color = match ty {
            ComponentType::Point => YELLOW.into(),
            ComponentType::Line => Color::NONE,
            ComponentType::Area => YELLOW.with_alpha(0.25).into(),
        };
        self
    }
    fn hover(&mut self, ty: ComponentType) -> &mut Self {
        self.color = match ty {
            ComponentType::Point => OLIVE.into(),
            ComponentType::Line => Color::NONE,
            ComponentType::Area => OLIVE.with_alpha(0.25).into(),
        };
        self
    }
}
impl HighlightExt for Stroke {
    fn select(&mut self, ty: ComponentType) -> &mut Self {
        self.color = match ty {
            ComponentType::Point => Color::NONE,
            ComponentType::Line | ComponentType::Area => YELLOW.into(),
        };
        self
    }
    fn hover(&mut self, ty: ComponentType) -> &mut Self {
        self.color = match ty {
            ComponentType::Point => Color::NONE,
            ComponentType::Line | ComponentType::Area => OLIVE.into(),
        };
        self
    }
}

fn style_in_max_zoom<T>(style: &HashMap<String, Vec<T>>) -> Option<&Vec<T>> {
    Some(
        style
            .iter()
            .map(|(zl, v)| (zl.split('-').next().unwrap().parse::<u8>().unwrap(), v))
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
