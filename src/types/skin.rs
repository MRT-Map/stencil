use std::collections::HashMap;
use crate::ComponentType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinInfo {
    pub size: u32,
    pub font: HashMap<String, String>,
    pub background: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "layer")]
pub enum SkinStyle {
    AreaFill {
        colour: String,
        outline: Option<String>,
        stripe: Option<(u32, u32, u8)>
    },
    AreaCenterText {
        colour: String,
        offset: (i32, i32),
        size: u32
    },
    AreaBorderText {
        colour: String,
        offset: (i32, i32),
        size: u32
    },
    LineFore {
        colour: String,
        width: u32,
        dash: Option<(u32, u32)>
    },
    LineBack {
        colour: String,
        width: u32,
        dash: Option<(u32, u32)>
    },
    LineText {
        colour: String,
        arrow_colour: String,
        size: u32,
        offset: i32
    },
    PointImage {
        file: String,
        offset: (i32, i32),
    },
    // TODO

}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinComponent {
    pub tags: Vec<String>,
    pub type_: ComponentType,
    pub style: HashMap<String, Vec<SkinStyle>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Skin {
    pub info: SkinInfo,
    pub order: Vec<String>,
    pub types: HashMap<String, SkinComponent>
}