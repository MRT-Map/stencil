use async_executor::{Executor, Task};
use bevy::prelude::{Commands, Local, NextState};
use egui_notify::ToastLevel;
use futures_lite::future;
use tracing::{error, info};

use crate::{
    component::skin::Skin,
    error::log::{AddToErrorLog, ErrorLogEntry, ERROR_LOG},
    misc::cache_path,
    state::LoadingState,
};

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
    mut executor: Local<Option<Executor>>,
) {
    if cache_path("skin.msgpack").exists() {
        if let Ok(skin) = std::fs::read(cache_path("skin.msgpack")) {
            if let Ok(skin) = rmp_serde::from_slice::<Skin>(&skin) {
                info!("Retrieved from cache");
                commands.insert_resource(skin);
                commands.insert_resource(NextState(Some(LoadingState::LoadSkin.next())));
                *task_s = Step::Complete;
            }
        }
    }
    let executor = executor.get_or_insert_with(Executor::new);
    match &mut *task_s {
        Step::Uninitialised => {
            let new_task = executor.spawn(async move {
                surf::get("https://raw.githubusercontent.com/MRT-Map/tile-renderer/main/renderer/skins/default.json")
                    .recv_json::<Skin>().await
            });
            info!("Retrieving skin");
            *task_s = Step::Pending(new_task);
        }
        Step::Pending(task) => match future::block_on(future::poll_once(task)) {
            None => {
                executor.try_tick();
            }
            Some(Ok(skin)) => {
                info!("Retrieved");
                let _ = rmp_serde::to_vec_named(&skin)
                    .map_err(color_eyre::Report::from)
                    .and_then(|s| {
                        std::fs::write(cache_path("skin.msgpack"), s)
                            .map_err(color_eyre::Report::from)
                    })
                    .add_to_error_log(ToastLevel::Warning);
                commands.insert_resource(skin);
                commands.insert_resource(NextState(Some(LoadingState::LoadSkin.next())));
                *task_s = Step::Complete;
            }
            Some(Err(err)) => {
                error!(?err, "Unable to retrieve skin");
                let mut error_log = ERROR_LOG.write().unwrap();
                error_log.pending_errors.push(ErrorLogEntry::new(&format!("Couldn't download skin.\nMake sure you are connected to the internet.\nError: {err}"), ToastLevel::Error));
                *task_s = Step::Complete;
            }
        },
        Step::Complete => {}
    }
}
