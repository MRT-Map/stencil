use std::{borrow::Cow, collections::HashSet, path::PathBuf, sync::Arc};

use eyre::{Report, eyre};
use pla::FullId;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use crate::{
    App,
    map::basemap::Basemap,
    notif,
    project::{Project, pla3::PlaComponent},
    utils::{
        file::safe_write,
        with_warnings::{WithWarning, WithWarnings},
    },
};

#[derive(Serialize, Deserialize)]
struct ProjectToml<'a> {
    pub basemap: Cow<'a, Basemap>,
    pub skin_url: Cow<'a, str>,
}

impl Project {
    #[tracing::instrument(skip_all)]
    pub fn load(path: PathBuf) -> eyre::Result<Self> {
        let project_toml_str = std::fs::read_to_string(path.join("project.toml"))
            .map_err(|e| eyre!("Cannot load project.toml in {}: {e:#}", path.display()))?;
        let project_toml: ProjectToml = toml::from_str(&project_toml_str)
            .map_err(|e| eyre!("Cannot parse project.toml in {}: {e:#}", path.display()))?;
        let mut s = Self {
            basemap: project_toml.basemap.into_owned(),
            skin_url: project_toml.skin_url.into_owned(),
            path: Some(path),
            ..Self::default()
        };
        let _ = s.update_namespace_list()?;
        Ok(s)
    }
    pub fn update_namespace_list(&mut self) -> eyre::Result<WithWarnings<()>> {
        let Some(path) = &self.path else {
            return Ok(WithWarnings::ok(()));
        };
        let mut errors = Vec::new();

        let mut folders = HashSet::new();
        for dir_entry in std::fs::read_dir(path)? {
            let Ok(dir_entry) = dir_entry.map_err(|e| errors.push(Report::from(e))) else {
                continue;
            };
            if let Ok(file_type) = dir_entry
                .file_type()
                .map_err(|e| errors.push(Report::from(e)))
                && file_type.is_dir()
            {
                folders.insert(dir_entry.file_name().to_string_lossy().into_owned());
            }
        }

        self.namespaces.retain(|namespace, loaded| {
            if folders.contains(namespace) {
                folders.remove(namespace);
                true
            } else {
                *loaded
            }
        });
        for namespace in folders {
            self.namespaces.insert(namespace, false);
        }

        Ok(WithWarnings::new((), errors))
    }
    #[tracing::instrument(skip_all)]
    pub fn load_namespace(&mut self, namespace: &str) -> eyre::Result<WithWarnings<()>> {
        let Some(path) = &self.path else {
            return Ok(WithWarnings::ok(()));
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
                FullId::new(namespace.to_owned(), id.to_string_lossy().into_owned()),
                |a| self.skin()?.get_type(a).map(Arc::clone),
            )
            .map(WithWarning::from)
            {
                Ok(ww) => {
                    let (component, _) = ww.handle_warning(|e| errors.push(e.into()));
                    self.components.insert(self.skin().unwrap(), component);
                }
                Err(e) => errors.push(e.into()),
            }
        }

        Ok(WithWarnings::new((), errors))
    }
    pub fn save_notif(&self) {
        if self.path.is_none() {
            return;
        }
        self.save().handle_warnings2(
            |errors| {
                notif!(warning "Errors while saving", errors &errors, "Errors while saving");
            },
            || {
                notif!(success "Saved project");
            },
        );
    }
    #[tracing::instrument(skip_all)]
    pub fn save(&self) -> WithWarnings<()> {
        let Some(path) = &self.path else {
            return WithWarnings::ok(());
        };
        let mut errors = Vec::new();

        let project_toml = ProjectToml {
            basemap: Cow::Borrowed(&self.basemap),
            skin_url: Cow::Borrowed(&self.skin_url),
        };
        if let Err(e) = toml::to_string_pretty(&project_toml)
            .map_err(Report::from)
            .and_then(|s| safe_write(path.join("project.toml"), s).map_err(Report::from))
        {
            errors.push(e);
        }

        errors.extend(self.save_components(self.components.iter()).warnings);

        WithWarnings::new((), errors)
    }
    pub fn save_components<'a, C: Iterator<Item = &'a PlaComponent>>(
        &self,
        components: C,
    ) -> WithWarnings<()> {
        let Some(path) = &self.path else {
            return WithWarnings::ok(());
        };
        let mut errors = Vec::new();

        for component in components {
            if let Err(e) = component
                .save_to_string(|ty| ty.name().as_str())
                .map_err(Report::from)
                .and_then(|s| safe_write(component.path(path), s).map_err(Report::from))
            {
                errors.push(e);
            }
        }

        WithWarnings::new((), errors)
    }
}

impl App {
    #[tracing::instrument(skip_all)]
    pub fn open_project(&mut self) {
        let Some(folder) = FileDialog::new().set_title("Open Project").pick_folder() else {
            return;
        };
        let project = match Project::load(folder) {
            Ok(p) => p,
            Err(e) => {
                notif!(error "Failed to load project", error &e);
                return;
            }
        };
        self.project = project;
    }
}
