use async_executor::{Executor, Task};
use bevy::prelude::*;
use egui_notify::ToastLevel;
use eyre::{eyre, OptionExt};
use futures_lite::future;
use semver::Version;

use crate::ui::notif::{NotifLogRwLockExt, NOTIF_LOG};

#[derive(Default)]
pub enum Step<T> {
    #[default]
    Uninitialised,
    Pending(Task<T>),
    Complete,
}

pub fn check_update_sy(
    mut task_s: Local<Step<Result<Version>>>,
    mut executor: Local<Option<Executor>>,
) -> Result {
    let executor = executor.get_or_insert_with(Executor::new);
    match &mut *task_s {
        Step::Uninitialised => {
            let new_task = executor.spawn(async move {
                let mut response =
                    surf::get("https://api.github.com/repos/mrt-map/stencil2/releases/latest")
                        .await
                        .map_err(|a| eyre!("{a:?}"))?;
                let release = response.body_json::<serde_json::Value>().await?;
                let tag_name = release
                    .get("tag_name")
                    .ok_or_eyre("No `tag_name`")?
                    .as_str()
                    .ok_or_eyre("`tag_name` not string")?;
                Ok(Version::parse(tag_name.trim_start_matches('v'))?)
            });
            info!("Querying latest version");
            *task_s = Step::Pending(new_task);
        }
        Step::Pending(task) => match future::block_on(future::poll_once(task)) {
            None => {
                executor.try_tick();
            }
            Some(Ok(latest)) => {
                info!(%latest, "Queried");
                let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
                if latest > current {
                    NOTIF_LOG.push(
                        format!("New version {latest} available (currently running {current})"),
                        ToastLevel::Warning,
                    );
                }
                *task_s = Step::Complete;
            }
            Some(Err(err)) => {
                error!(?err, "Unable to retrieve latest version");
                NOTIF_LOG.push(
                    format!("Couldn't get latest Stencil2 version\nError: {err}"),
                    ToastLevel::Warning,
                );
                *task_s = Step::Complete;
            }
        },
        Step::Complete => {}
    }
    Ok(())
}

pub struct UpdateCheckerPlugin;

impl Plugin for UpdateCheckerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_update_sy);
    }
}
