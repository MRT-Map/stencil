use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64_serde::base64_serde_type;
use bevy::prelude::*;
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
            Self::Point { name, .. } => name,
            Self::Line { name, .. } => name,
            Self::Area { name, .. } => name,
        }
    }
    #[must_use]
    pub const fn tags(&self) -> &Vec<String> {
        match self {
            Self::Point { tags, .. } => tags,
            Self::Line { tags, .. } => tags,
            Self::Area { tags, .. } => tags,
        }
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
    pub prune_small_text: Option<f64>,
    pub licence: String,
}

impl Skin {
    #[must_use]
    pub fn get_type(&self, ty: &str) -> Option<&SkinComponent> {
        self.types.iter().find(|a| a.name() == ty)
    }
    #[must_use]
    pub fn get_order(&self, ty: &str) -> Option<usize> {
        self.types.iter().position(|a| a.name() == ty)
    }
}
