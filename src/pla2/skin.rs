use std::{collections::HashMap, sync::Arc};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use hex_color::HexColor;
use iyes_loopless::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{misc::EditorState, pla2::component::ComponentType, ui::popup::Popup};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SkinInfo {
    pub size: u32,
    pub font: HashMap<String, String>,
    pub background: HexColor,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "layer")]
pub enum AreaStyle {
    #[serde(rename = "fill")]
    Fill {
        colour: Option<HexColor>,
        outline: Option<HexColor>,
        stripe: Option<(u32, u32, u8)>,
    },
    #[serde(rename = "centertext")]
    CenterText {
        colour: HexColor,
        offset: IVec2,
        size: u32,
    },
    #[serde(rename = "bordertext")]
    BorderText {
        colour: HexColor,
        offset: i32,
        size: u32,
    },
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "layer")]
pub enum LineStyle {
    #[serde(rename = "fore")]
    Fore {
        colour: HexColor,
        width: u32,
        dash: Option<UVec2>,
    },
    #[serde(rename = "back")]
    Back {
        colour: HexColor,
        width: u32,
        dash: Option<UVec2>,
    },
    #[serde(rename = "text")]
    Text {
        colour: HexColor,
        arrow_colour: HexColor,
        size: u32,
        offset: i32,
    },
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "layer")]
pub enum PointStyle {
    #[serde(rename = "image")]
    Image { file: String, offset: IVec2 },
    #[serde(rename = "square")]
    Square {
        colour: HexColor,
        outline: Option<HexColor>,
        size: u32,
        width: u32,
    },
    #[serde(rename = "circle")]
    Circle {
        colour: HexColor,
        outline: Option<HexColor>,
        size: u32,
        width: u32,
    },
    #[serde(rename = "text")]
    Text {
        colour: HexColor,
        size: u32,
        offset: IVec2,
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
impl SkinComponent {
    pub fn get_type(&self) -> ComponentType {
        match self {
            Self::Point { .. } => ComponentType::Point,
            Self::Line { .. } => ComponentType::Line,
            Self::Area { .. } => ComponentType::Area,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Skin {
    pub info: SkinInfo,
    pub order: Vec<String>,
    pub types: HashMap<String, SkinComponent>,
}

#[derive(Default)]
pub enum Step<T> {
    #[default]
    Uninitialised,
    Pending(Task<T>),
    Complete,
}

pub fn get_skin_sy(
    mut commands: Commands,
    mut task_s: Local<Step<surf::Result<Skin>>>,
    mut popup: EventWriter<Arc<Popup>>,
) {
    match &mut *task_s {
        Step::Uninitialised => {
            let thread_pool = AsyncComputeTaskPool::get();
            let new_task = thread_pool.spawn(async move {
                surf::get("https://raw.githubusercontent.com/MRT-Map/tile-renderer/main/renderer/skins/default.json")
                    .recv_json::<Skin>().await
            });
            info!("Retrieving skin");
            *task_s = Step::Pending(new_task);
        }
        Step::Pending(task) => match future::block_on(future::poll_once(task)) {
            None => {}
            Some(Ok(skin)) => {
                info!("Retrieved");
                commands.insert_resource(skin);
                commands.insert_resource(NextState(EditorState::Idle));
                *task_s = Step::Complete;
            }
            Some(Err(err)) => {
                error!(?err, "Unable to retrieve skin");
                popup.send(Popup::base_alert(
                    "quit1",
                    "Unable to load skin",
                    format!("Make sure you are connected to the internet.\nError: {err}"),
                ));
                *task_s = Step::Complete;
            }
        },
        Step::Complete => {}
    }
}
