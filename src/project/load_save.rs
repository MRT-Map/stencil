use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Cursor, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use eyre::{Report, eyre};
use pla::FullId;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    App,
    component_actions::event::ComponentEv,
    map::basemap::Basemap,
    notif,
    project::{Project, history::Events, namespace_event::NamespaceEv, pla3::PlaComponent},
    ui::popup::Popup,
    utils::{
        file::safe_write,
        with_warnings::{ErrorWarningsExt, WithWarning, WithWarnings},
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
            .map_err(|e| eyre!("Cannot load project.toml at {}: {e:#}", path.display()))?;
        let project_toml: ProjectToml = toml::from_str(&project_toml_str).map_err(|e| {
            eyre!(
                "Cannot deserialise project.toml at {}: {e:#}",
                path.display()
            )
        })?;
        let mut s = Self {
            basemap: project_toml.basemap.into_owned(),
            skin_url: project_toml.skin_url.into_owned(),
            path: Some(path),
            namespaces: HashMap::default(),
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
            let Some(component) = PlaComponent::load(
                &string,
                FullId::new(namespace.to_owned(), id.to_string_lossy().into_owned()),
                |a| self.skin()?.get_type(a).map(Arc::clone),
            )
            .map(WithWarning::from)
            .error_warnings_to_vec(&mut errors) else {
                continue;
            };
            self.components.insert(self.skin().unwrap(), component);
        }

        Ok(WithWarnings::new((), errors))
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
            .map_err(|e| eyre!("Cannot serialise project.toml: {e:#}"))
            .and_then(|s| {
                safe_write(path.join("project.toml"), s)
                    .map_err(|e| eyre!("Cannot write project.toml to {}: {e:#}", path.display()))
            })
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

    #[tracing::instrument(skip_all)]
    pub fn import_namespace_pla3_zip(
        &self,
        path: &Path,
    ) -> eyre::Result<WithWarnings<(String, Vec<Events>)>> {
        let mut errors = Vec::new();
        let mut events = Vec::<Events>::new();
        let Some(namespace) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".pla3.zip"))
        else {
            return Err(eyre!("File `{}` must end with `.pla3.zip`", path.display()));
        };
        if namespace.is_empty() {
            return Err(eyre!("Namespace name must not be empty"));
        }
        if !self.namespaces.contains_key(namespace) {
            events.push(NamespaceEv::Create(namespace.to_owned()).into());
        }

        let mut archive = ZipArchive::new(File::open(path)?)?;
        let mut components = Vec::new();
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let enclosed_name = file.enclosed_name();
            let Some(id) = enclosed_name
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".pla3"))
            else {
                errors.push(eyre!("Invalid file path {}", path.display()));
                continue;
            };

            let full_id = FullId::new(namespace.to_owned(), id.to_owned());
            if self.components.iter().any(|a| a.full_id == full_id) {
                errors.push(eyre!("ID `{full_id}` already exists"));
                continue;
            }

            let Some(component) = PlaComponent::load_from_buf(BufReader::new(file), full_id, |a| {
                self.skin()?.get_type(a).map(Arc::clone)
            })
            .map(WithWarning::from)
            .error_warnings_to_vec(&mut errors) else {
                continue;
            };
            components.push(component);
        }
        events.push(ComponentEv::Create(components).into());

        Ok(WithWarnings::new((namespace.into(), events), errors))
    }

    #[tracing::instrument(skip_all)]
    pub fn export_namespace_pla3_zip(&self, namespace: &str, path: &Path) -> eyre::Result<()> {
        let mut cursor = Cursor::new(Vec::new());
        let mut archive = ZipWriter::new(&mut cursor);

        for component in self.components.iter_namespace(namespace) {
            let string = component.save_to_string(|ty| ty.name().as_str())?;
            archive.start_file_from_path(component.file_name(), SimpleFileOptions::default())?;
            archive.write_all(string.as_bytes())?;
        }

        archive.finish()?;
        safe_write(path, cursor.into_inner())?;

        Ok(())
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
    #[tracing::instrument(skip_all)]
    pub fn reload_project(&mut self) {
        if self.project.path.is_none() {
            return;
        }
        let Ok(()) = self.project.update_namespace_list().notify(
            "Error while reloading project",
            "Errors while reloading project",
        ) else {
            return;
        };
        notif!(success "Reloaded project");
    }
    #[tracing::instrument(skip_all)]
    pub fn save_project(&mut self) {
        if self.project.path.is_none() {
            return self.save_project_as();
        }
        self.project.save().notify("Errors while saving project");
        notif!(success "Saved project");
    }
    #[tracing::instrument(skip_all)]
    pub fn save_project_as(&mut self) {
        let Some(folder) = FileDialog::new().set_title("Save Project As").pick_folder() else {
            return;
        };
        self.project.path = Some(folder);
        self.save_project();
    }

    #[tracing::instrument(skip_all)]
    pub fn import_namespace_pla3_zip(&mut self) {
        let Some(files) = FileDialog::new()
            .set_title("Import pla3.zip")
            .add_filter("pla3.zip file", &["pla3.zip"])
            .pick_files()
        else {
            return;
        };
        for file in files {
            let Ok((namespace, events)) = self
                .project
                .import_namespace_pla3_zip(&file)
                .notify("Error while importing", "Errors while importing")
            else {
                continue;
            };
            for event in events {
                self.run_event(event);
            }
            notif!(success format!("Imported `{namespace}`"));
        }
    }
    #[tracing::instrument(skip_all)]
    pub fn export_namespaces_pla3_zip(&mut self) {
        self.add_popup(ChooseNamespacesPopup::default());
    }

    #[tracing::instrument(skip_all)]
    pub fn export_namespace_pla3_zip(&mut self, namespace: &str) {
        let Some(file) = FileDialog::new()
            .set_title("Export pla3.zip")
            .set_file_name(format!("{namespace}.pla3.zip"))
            .add_filter("pla3.zip file", &["pla3.zip"])
            .save_file()
        else {
            return;
        };
        match self.project.export_namespace_pla3_zip(namespace, &file) {
            Ok(()) => {
                notif!(success format!("Exported to {}", file.display()));
            }
            Err(e) => {
                notif!(error "Errors while exporting", error &e);
            }
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct ChooseNamespacesPopup {
    selected: HashSet<String>,
}

impl Popup for ChooseNamespacesPopup {
    fn id(&self) -> String {
        "choose-namespaces".into()
    }

    fn title(&self) -> String {
        "Choose Namespaces".into()
    }

    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) -> bool {
        for (namespace, _) in app.project.namespaces.iter().filter(|(_, v)| **v) {
            let mut checked = self.selected.contains(namespace);
            if ui.checkbox(&mut checked, namespace).changed() {
                if checked {
                    self.selected.insert(namespace.to_owned());
                } else {
                    self.selected.remove(namespace);
                }
            }
        }
        let (cancel, r#continue) = ui
            .horizontal(|ui| {
                (
                    ui.button("Cancel").clicked(),
                    ui.button("Continue").clicked(),
                )
            })
            .inner;
        if !cancel && !r#continue {
            return true;
        }
        if cancel {
            return false;
        }
        for namespace in &self.selected {
            app.export_namespace_pla3_zip(namespace);
        }
        false
    }
}
