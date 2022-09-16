use std::collections::HashMap;
use bevy::app::AppExit;
use futures_lite::future;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use iyes_loopless::prelude::*;
use native_dialog::{MessageDialog, MessageType};
use serde::{Deserialize, Serialize};

use crate::types::{ComponentType, EditorState};

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

#[derive(Component)]
pub struct AsyncTask<T>(Task<T>);

pub fn request_skin(
    mut commands: Commands
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        surf::get("https://raw.githubusercontent.com/MRT-Map/tile-renderer/main/renderer/skins/default.json")
            .recv_json::<Skin>().await
    });
    info!("Retrieving skin");
    commands.spawn().insert(AsyncTask(task));
}

pub fn retrieve_skin(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut AsyncTask<surf::Result<Skin>>)>,
    mut exit: EventWriter<AppExit>
) {
    for (entity, mut task) in tasks.iter_mut() {
        match future::block_on(future::poll_once(&mut task.0)) {
            None => {},
            Some(Ok(skin)) => {
                info!("Retrieved");
                commands.insert_resource(skin);
                commands.insert_resource(NextState(EditorState::Idle));
                commands.entity(entity).despawn();
            }
            Some(Err(err)) => {
                error!(?err, "Unable to retrieve skin");
                MessageDialog::new()
                    .set_type(MessageType::Error)
                    .set_title("Unable to load skin, make sure you are connected to the internet.")
                    .set_text(&*format!("Error: {:?}", err))
                    .show_alert().unwrap();
                exit.send(AppExit)
            }
        }

    }
}
