use std::sync::Arc;

use bevy::{
    prelude::{Commands, EventWriter, Local, NextState},
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use tracing::{error, info};

use crate::{pla2::skin::Skin, state::LoadingState, ui::popup::Popup};

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
    mut popup: EventWriter<Popup>,
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
                commands.insert_resource(NextState(Some(LoadingState::LoadSkin.next())));
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
