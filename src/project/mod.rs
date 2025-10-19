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
use tracing::error;

use crate::{
    EXECUTOR,
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
    pub fn load_skin(&mut self) {
        match &mut self.skin_status {
            SkinStatus::Unloaded => {
                let skin_url = self.skin_url.clone();
                let task = EXECUTOR.spawn(async move { surf::get(skin_url).recv_json().await });
                self.skin_status = SkinStatus::Loading(task);
            }
            SkinStatus::Loading(task) => match future::block_on(future::poll_once(task)) {
                Some(Ok(skin)) => {
                    self.skin_status = SkinStatus::Loaded(skin);
                }
                Some(Err(e)) => {
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
}
