pub mod component_editor;
pub mod component_list;
pub mod history;
pub mod history_viewer;
pub mod load_save;
pub mod namespace_event;
pub mod pla3;
pub mod project_editor;
pub mod skin;

use std::path::PathBuf;

use async_executor::Task;
use egui::ahash::HashMap;
use etcetera::AppStrategy;
use eyre::{Report, eyre};
use futures_lite::future;
use history::History;
use pla::Namespace;
use tracing::{error, info};

use crate::{
    map::basemap::Basemap,
    project::{component_list::ComponentList, skin::Skin},
    utils::{
        EXECUTOR, PATH_SANITISER,
        file::{FOLDERS, safe_write},
    },
};

#[derive(Debug, Default)]
pub enum SkinStatus {
    #[default]
    Unloaded,
    Loading(Task<eyre::Result<Skin>>),
    Failed(Report),
    Loaded(&'static Skin),
}

pub struct Project {
    pub basemap: Basemap,
    pub skin_status: SkinStatus,
    pub skin_url: String,
    pub components: ComponentList,
    pub namespaces: HashMap<Namespace, bool>,
    pub new_component_ns: Option<Namespace>,
    pub path: Option<PathBuf>,
    pub history: History,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            basemap: Basemap::default(),
            skin_status: SkinStatus::default(),
            skin_url: "https://github.com/MRT-Map/tile-renderer/releases/latest/download/default.nofontfiles.skin.json".into(),
            components: ComponentList::default(),
            namespaces: HashMap::from_iter([(Namespace::default(), true)]),
            new_component_ns: None,
            path: None,
            history: History::default(),
        }
    }
}

impl Project {
    pub const fn skin(&self) -> Option<&'static Skin> {
        match &self.skin_status {
            SkinStatus::Loaded(skin) => Some(skin),
            _ => None,
        }
    }
    pub fn skin_cache_path(&self) -> PathBuf {
        FOLDERS
            .in_cache_dir("skin")
            .join(PATH_SANITISER.replace_all(&self.skin_url, "").as_ref())
    }
    #[tracing::instrument(skip_all)]
    pub fn load_skin(&mut self, ctx: &egui::Context) {
        match &mut self.skin_status {
            SkinStatus::Unloaded => {
                let skin_cache = self.skin_cache_path();
                if skin_cache.exists()
                    && let Ok(s) = std::fs::read_to_string(&skin_cache)
                        .inspect_err(|e| error!(?skin_cache, "Cannot read skin cache: {e:#}"))
                    && let Ok(skin) = serde_json::from_str(&s).inspect_err(|e| {
                        error!(?skin_cache, "Cannot deserialise skin cache: {e:#}");
                    })
                {
                    info!(?skin_cache, "Loaded skin cache");
                    self.skin_status = SkinStatus::Loaded(Box::leak(skin));
                    return;
                }

                let skin_url = self.skin_url.clone();
                info!(skin_url, "Loading skin");
                let task = EXECUTOR.spawn(async move {
                    Ok(ehttp::fetch_async(ehttp::Request::get(skin_url))
                        .await
                        .map_err(Report::msg)?
                        .json()?)
                });
                self.skin_status = SkinStatus::Loading(task);
            }
            SkinStatus::Loading(task) => match future::block_on(future::poll_once(task)) {
                Some(Ok(mut skin)) => {
                    skin.setup_order_cache();
                    info!("Skin loaded");

                    let skin_cache = self.skin_cache_path();
                    if let Ok(s) = serde_json::to_string(&skin).inspect_err(|e| {
                        error!(self.skin_url, "Cannot serialise skin cache: {e:#}");
                    }) && safe_write(&skin_cache, &s)
                        .inspect_err(|e| error!(?skin_cache, "Cannot write skin cache: {e:#}"))
                        .is_ok()
                    {
                        info!(?skin_cache, "Wrote skin to cache file");
                    }

                    self.skin_status = SkinStatus::Loaded(Box::leak(skin.into()));
                }
                Some(Err(e)) => {
                    error!("Skin failed to load: {e:#}");
                    self.skin_status = SkinStatus::Failed(e);
                }
                None => {
                    ctx.request_repaint_after_secs(1.0);
                }
            },
            _ => {}
        }
    }
    pub fn namespace_component_count(&self, namespace: &Namespace) -> eyre::Result<usize> {
        if self.namespaces.get(namespace).is_some_and(|a| *a) {
            return Ok(self.components.iter_namespace(namespace).count());
        }
        let Some(path) = &self.path else {
            return Err(eyre!("scratchpad contains unloaded namespace"));
        };
        Ok(std::fs::read_dir(path.join(namespace))?
            .filter_map(Result::ok)
            .filter(|a| a.file_type().is_ok_and(|a| !a.is_dir()))
            .filter(|a| a.path().extension() == Some("pla3".as_ref()))
            .count())
    }
}
