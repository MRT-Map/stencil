pub mod pla3;
pub mod skin;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use async_executor::Task;
use eyre::{Report, Result};
use futures_lite::future;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    EXECUTOR, URL_REPLACER,
    file::cache_dir,
    map::{basemap::Basemap, tile_coord::TILE_CACHE},
    project::{pla3::PlaComponent, skin::Skin},
};

#[derive(Debug, Default)]
pub enum SkinStatus {
    #[default]
    Unloaded,
    Loading(Task<surf::Result<Skin>>),
    Failed(surf::Error),
    Loaded(Skin),
}

#[derive(Debug)]
pub struct Project {
    pub basemap: Basemap,
    pub skin_status: SkinStatus,
    pub skin_url: String,
    pub components: Vec<PlaComponent>,
    pub path: Option<PathBuf>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            basemap: Basemap::default(),
            skin_status: SkinStatus::default(),
            skin_url: "https://github.com/MRT-Map/tile-renderer/releases/latest/download/default.nofontfiles.skin.json".into(),
            components: Vec::new(),
            path: None,
        }
    }
}

impl Project {
    pub const fn skin(&self) -> Option<&Skin> {
        match &self.skin_status {
            SkinStatus::Loaded(skin) => Some(skin),
            _ => None,
        }
    }
    pub fn skin_cache_path(&self) -> PathBuf {
        cache_dir("skin").join(format!(
            "{}.json",
            URL_REPLACER.replace_all(&self.skin_url, "")
        ))
    }
    pub fn load_skin(&mut self) {
        match &mut self.skin_status {
            SkinStatus::Unloaded => {
                let skin_cache = self.skin_cache_path();
                if skin_cache.exists()
                    && let Ok(s) = std::fs::read_to_string(&skin_cache)
                        .inspect_err(|e| error!(?skin_cache, "Cannot read skin cache: {e:?}"))
                    && let Ok(skin) = serde_json::from_str(&s).inspect_err(|e| {
                        error!(?skin_cache, "Cannot deserialise skin cache: {e:?}")
                    })
                {
                    info!(?skin_cache, "Loaded skin cache");
                    self.skin_status = SkinStatus::Loaded(skin);
                    return;
                }

                let skin_url = self.skin_url.clone();
                info!(skin_url, "Loading skin");
                let task = EXECUTOR.spawn(async move {
                    surf::get(skin_url)
                        .middleware(surf::middleware::Redirect::default())
                        .recv_json()
                        .await
                });
                self.skin_status = SkinStatus::Loading(task);
            }
            SkinStatus::Loading(task) => match future::block_on(future::poll_once(task)) {
                Some(Ok(skin)) => {
                    info!("Skin loaded");

                    let skin_cache = self.skin_cache_path();
                    if let Ok(s) = serde_json::to_string(&skin).inspect_err(|e| {
                        error!(self.skin_url, "Cannot serialise skin cache: {e:?}")
                    }) && std::fs::write(&skin_cache, &s)
                        .inspect_err(|e| error!(?skin_cache, "Cannot write skin cache: {e:?}"))
                        .is_ok()
                    {
                        info!(?skin_cache, "Wrote skin to cache file");
                    }

                    self.skin_status = SkinStatus::Loaded(skin);
                }
                Some(Err(e)) => {
                    error!("Skin failed to load: {e:?}");
                    self.skin_status = SkinStatus::Failed(e);
                }
                None => {}
            },
            _ => {}
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ProjectToml<'a> {
    pub basemap: Cow<'a, Basemap>,
    pub skin_url: Cow<'a, str>,
}

impl Project {
    pub fn load(path: PathBuf) -> Result<Self> {
        let project_toml: ProjectToml =
            toml::from_str(&std::fs::read_to_string(path.join("project.toml"))?)?;
        Ok(Self {
            basemap: project_toml.basemap.into_owned(),
            skin_status: SkinStatus::default(),
            skin_url: project_toml.skin_url.into_owned(),
            components: Vec::new(),
            path: Some(path),
        })
    }
    pub fn load_namespace(&mut self, namespace: &str) -> Result<Vec<Report>> {
        let Some(path) = &self.path else {
            return Ok(Vec::new());
        };
        let mut errors = Vec::new();

        for dir_entry in std::fs::read_dir(path.join(namespace))? {
            let Ok(dir_entry) = dir_entry.map_err(|e| errors.push(Report::from(e))) else {
                continue;
            };
            let file_path = dir_entry.path();
            if file_path.extension() != Some("pla3".as_ref()) {
                continue;
            }
            let Ok(string) =
                std::fs::read_to_string(file_path).map_err(|e| errors.push(Report::from(e)))
            else {
                continue;
            };
            let Some(id) = path.file_prefix() else {
                continue;
            };
            match PlaComponent::load_from_string(
                &string,
                namespace.to_owned(),
                id.to_string_lossy().into_owned(),
                self,
            ) {
                Ok((component, unknown_type_error)) => {
                    if let Some(e) = unknown_type_error {
                        errors.push(e);
                    }
                    self.components.push(component);
                }
                Err(e) => errors.push(e),
            }
        }

        Ok(errors)
    }
    pub fn save(&self) -> Vec<Report> {
        let Some(path) = &self.path else {
            return Vec::new();
        };
        let mut errors = Vec::new();

        let project_toml = ProjectToml {
            basemap: Cow::Borrowed(&self.basemap),
            skin_url: Cow::Borrowed(&self.skin_url),
        };
        if let Err(e) = toml::to_string_pretty(&project_toml)
            .map_err(Report::from)
            .and_then(|s| std::fs::write(path.join("project.toml"), s).map_err(Report::from))
        {
            errors.push(e);
        }

        errors.extend(self.save_components(&self.components));

        errors
    }
    pub fn save_components(&self, components: &[PlaComponent]) -> Vec<Report> {
        let Some(path) = &self.path else {
            return Vec::new();
        };
        let mut errors = Vec::new();

        for component in components {
            if let Err(e) = component
                .save_to_string()
                .and_then(|s| std::fs::write(component.path(path), s).map_err(Report::from))
            {
                errors.push(e);
            }
        }

        errors
    }
    pub fn namespace_component_count(&self, namespace: &str) -> Result<usize> {
        let Some(path) = &self.path else {
            return Ok(self
                .components
                .iter()
                .filter(|a| a.namespace == namespace)
                .count());
        };
        Ok(std::fs::read_dir(path.join(namespace))?
            .filter_map(Result::ok)
            .filter(|a| a.file_type().is_ok_and(|a| !a.is_dir()))
            .filter(|a| a.path().extension() == Some("pla3".as_ref()))
            .count())
    }
}
