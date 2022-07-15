use crate::EditorState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SkinInfo {
    pub size: u32,
    pub font: HashMap<String, String>,
    pub background: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "layer")]
pub enum AreaStyle {
    #[serde(rename = "fill")]
    Fill {
        colour: Option<String>,
        outline: Option<String>,
        stripe: Option<(u32, u32, u8)>,
    },
    #[serde(rename = "centertext")]
    CenterText {
        colour: String,
        offset: (i32, i32),
        size: u32,
    },
    #[serde(rename = "bordertext")]
    BorderText {
        colour: String,
        offset: i32,
        size: u32,
    },
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "layer")]
pub enum LineStyle {
    #[serde(rename = "fore")]
    Fore {
        colour: String,
        width: u32,
        dash: Option<(u32, u32)>,
    },
    #[serde(rename = "back")]
    Back {
        colour: String,
        width: u32,
        dash: Option<(u32, u32)>,
    },
    #[serde(rename = "text")]
    Text {
        colour: String,
        arrow_colour: String,
        size: u32,
        offset: i32,
    },
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "layer")]
pub enum PointStyle {
    #[serde(rename = "image")]
    Image { file: String, offset: (i32, i32) },
    #[serde(rename = "square")]
    Square {
        colour: String,
        outline: Option<String>,
        size: u32,
        width: u32,
    },
    #[serde(rename = "circle")]
    Circle {
        colour: String,
        outline: Option<String>,
        size: u32,
        width: u32,
    },
    #[serde(rename = "text")]
    Text {
        colour: String,
        size: u32,
        offset: (i32, i32),
        anchor: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SkinComponent {
    #[serde(rename = "point")]
    Point {
        tags: Vec<String>,
        style: HashMap<String, Vec<PointStyle>>,
    },
    #[serde(rename = "line")]
    Line {
        tags: Vec<String>,
        style: HashMap<String, Vec<LineStyle>>,
    },
    #[serde(rename = "area")]
    Area {
        tags: Vec<String>,
        style: HashMap<String, Vec<AreaStyle>>,
    },
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Skin {
    pub info: SkinInfo,
    pub order: Vec<String>,
    pub types: HashMap<String, SkinComponent>,
}

pub fn get_skin(mut commands: Commands) {
    let skin = reqwest::blocking::get(
        "https://raw.githubusercontent.com/MRT-Map/tile-renderer/main/renderer/skins/default.json",
    )
    .unwrap()
    .json::<Skin>()
    .unwrap();
    commands.insert_resource(skin);
    commands.insert_resource(NextState(EditorState::Idle));
}
