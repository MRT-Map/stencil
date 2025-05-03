use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64_serde::base64_serde_type;
use bevy::prelude::*;
use bevy_egui::egui;
use hex_color::HexColor;
use serde::{Deserialize, Serialize};

use crate::component::pla2::ComponentType;

base64_serde_type!(Base64Standard, STANDARD);

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "ty")]
pub enum AreaStyle {
    #[serde(rename = "areaFill")]
    Fill {
        zoom_multiplier: f32,
        colour: Option<HexColor>,
        outline: Option<HexColor>,
        outline_width: f32,
    },
    #[serde(rename = "areaCentreText")]
    CenterText {
        zoom_multiplier: f32,
        colour: Option<HexColor>,
        offset: Vec2,
        size: f32,
    },
    #[serde(rename = "areaBorderText")]
    BorderText {
        zoom_multiplier: f32,
        colour: Option<HexColor>,
        offset: f32,
        size: f32,
    },
    #[serde(rename = "areaCentreImage")]
    CentreImage {
        zoom_multiplier: f32,
        #[serde(with = "Base64Standard")]
        image: Vec<u8>,
        extension: String,
        size: Vec2,
        offset: Vec2,
    },
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "ty")]
pub enum LineStyle {
    #[serde(rename = "lineFore")]
    Fore {
        zoom_multiplier: f32,
        colour: Option<HexColor>,
        width: f32,
        dash: Option<Vec<f32>>,
        unrounded: bool,
    },
    #[serde(rename = "lineBack")]
    Back {
        zoom_multiplier: f32,
        colour: Option<HexColor>,
        width: f32,
        dash: Option<Vec<f32>>,
        unrounded: bool,
    },
    #[serde(rename = "lineText")]
    Text {
        zoom_multiplier: f32,
        colour: Option<HexColor>,
        arrow_colour: Option<HexColor>,
        size: f32,
        offset: f32,
    },
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "ty")]
pub enum PointStyle {
    #[serde(rename = "pointImage")]
    Image {
        zoom_multiplier: f32,
        #[serde(with = "Base64Standard")]
        image: Vec<u8>,
        extension: String,
        size: Vec2,
        offset: Vec2,
    },
    #[serde(rename = "pointSquare")]
    Square {
        zoom_multiplier: f32,
        colour: Option<HexColor>,
        border_radius: f32,
        size: f32,
        width: f32,
    },
    #[serde(rename = "pointText")]
    Text {
        zoom_multiplier: f32,
        colour: Option<HexColor>,
        size: f32,
        offset: Vec2,
        anchor: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "shape")]
pub enum SkinComponent {
    #[serde(rename = "point")]
    Point {
        name: String,
        tags: Vec<String>,
        styles: HashMap<String, Vec<PointStyle>>,
    },
    #[serde(rename = "line")]
    Line {
        name: String,
        tags: Vec<String>,
        styles: HashMap<String, Vec<LineStyle>>,
    },
    #[serde(rename = "area")]
    Area {
        name: String,
        tags: Vec<String>,
        styles: HashMap<String, Vec<AreaStyle>>,
    },
}
impl SkinComponent {
    #[must_use]
    pub const fn get_type(&self) -> ComponentType {
        match self {
            Self::Point { .. } => ComponentType::Point,
            Self::Line { .. } => ComponentType::Line,
            Self::Area { .. } => ComponentType::Area,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &String {
        match self {
            Self::Point { name, .. } | Self::Line { name, .. } | Self::Area { name, .. } => name,
        }
    }
    #[must_use]
    pub const fn tags(&self) -> &Vec<String> {
        match self {
            Self::Point { tags, .. } | Self::Line { tags, .. } | Self::Area { tags, .. } => tags,
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

    #[must_use]
    pub fn front_colour(&self) -> Option<&HexColor> {
        match self {
            Self::Point { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    PointStyle::Square { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
            Self::Line { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Fore { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
            Self::Area { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
        }
    }
    #[must_use]
    pub fn back_colour(&self) -> Option<&HexColor> {
        match self {
            Self::Point { .. } => None,
            Self::Line { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Back { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
            Self::Area { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { outline, .. } => outline.into(),
                    _ => None,
                })
                .next_back(),
        }
    }
    #[must_use]
    pub fn text_colour(&self) -> Option<&HexColor> {
        match self {
            Self::Point { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    PointStyle::Text { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
            Self::Line { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Text { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
            Self::Area { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::CenterText { colour, .. } => colour.into(),
                    _ => None,
                })
                .next_back(),
        }
    }
    #[must_use]
    pub fn weight(&self) -> Option<f32> {
        match self {
            Self::Point { .. } => None,
            Self::Line { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    LineStyle::Fore { width, .. } => Some(*width),
                    _ => None,
                })
                .next_back(),
            Self::Area { styles, .. } => Self::style_in_max_zoom(styles)?
                .iter()
                .filter_map(|style| match style {
                    AreaStyle::Fill { outline_width, .. } => Some(outline_width * 5.0),
                    _ => None,
                })
                .next_back(),
        }
    }

    #[must_use]
    pub fn widget_text(
        &self,
        ui: &mut egui::Ui,
        text_style: &egui::TextStyle,
    ) -> impl Into<egui::WidgetText> {
        let font_id = &ui.style().text_styles[text_style];
        let mut label = egui::text::LayoutJob::default();
        let space = if let Some(c) = self.front_colour() {
            label.append(
                "◼",
                0.0,
                egui::TextFormat {
                    font_id: font_id.to_owned(),
                    color: egui::Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a),
                    ..default()
                },
            );
            font_id.size / 4.0
        } else if let Some(c) = self.back_colour() {
            label.append(
                "□",
                0.0,
                egui::TextFormat {
                    font_id: font_id.to_owned(),
                    color: egui::Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a),
                    ..default()
                },
            );
            font_id.size / 4.0
        } else {
            0.0
        };
        label.append(
            self.name(),
            space,
            egui::TextFormat {
                font_id: font_id.to_owned(),
                ..default()
            },
        );
        label
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Resource)]
pub struct Skin {
    pub version: u8,
    pub name: String,
    pub types: Vec<SkinComponent>,
    pub font_files: Vec<(String, String)>,
    pub font_string: String,
    pub background: HexColor,
    pub prune_small_text: Option<f32>,
    pub licence: String,
}

impl Skin {
    #[must_use]
    pub fn get_type(&self, ty: &str) -> Option<&SkinComponent> {
        self.types.iter().find(|a| a.name() == ty)
    }
    #[must_use]
    pub fn show_type(
        &self,
        ty: &str,
        ui: &mut egui::Ui,
        text_style: &egui::TextStyle,
    ) -> impl Into<egui::WidgetText> {
        self.get_type(ty).map_or_else(
            || egui::WidgetText::from(ty),
            |a| a.widget_text(ui, text_style).into(),
        )
    }
    #[must_use]
    pub fn get_order(&self, ty: &str) -> Option<usize> {
        self.types.iter().position(|a| a.name() == ty)
    }
}
