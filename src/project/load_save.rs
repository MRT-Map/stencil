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
    map::basemap::Basemap,
    notif,
    project::{Project, pla3::PlaComponent},
    ui::popup::Popup,
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
            match PlaComponent::load(
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
    pub fn import_namespace_pla3_zip(&mut self, path: &Path) -> eyre::Result<WithWarnings<()>> {
        let mut errors = Vec::new();
        let Some(namespace) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".pla3.zip"))
        else {
            return Err(eyre!("File must end with `.pla3.zip`"));
        };
        if namespace.is_empty() {
            return Err(eyre!("File name must not be empty"));
        }

        let mut archive = ZipArchive::new(File::open(path)?)?;
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

            match PlaComponent::load_from_buf(
                BufReader::new(file),
                FullId::new(namespace.to_owned(), id.to_owned()),
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

        self.namespaces.insert(namespace.to_owned(), true);

        Ok(WithWarnings::new((), errors))
    }

    #[tracing::instrument(skip_all)]
    pub fn export_namespace_pla3_zip(&self, namespace: &str, path: &Path) -> eyre::Result<()> {
        let mut cursor = Cursor::new(Vec::new());
        let mut archive = ZipWriter::new(&mut cursor);
        let components = self
            .components
            .iter()
            .filter(|a| a.full_id.namespace == namespace);

        for component in components {
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
        match self.project.update_namespace_list() {
            Ok(ww) => {
                ww.handle_warnings(|errors| {
                    notif!(warning "Errors while reloading project", errors &errors);
                });
                notif!(success "Reloaded project");
            }
            Err(e) => {
                notif!(error "Error while reloading project`", error &e);
            }
        }
    }
    #[tracing::instrument(skip_all)]
    pub fn save_project(&mut self) {
        if self.project.path.is_none() {
            return self.save_project_as();
        }
        self.project.save().handle_warnings2(
            |errors| {
                notif!(warning "Errors while saving", errors &errors);
            },
            || {
                notif!(success "Saved project");
            },
        );
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
            match self.project.import_namespace_pla3_zip(&file) {
                Ok(ww) => {
                    ww.handle_warnings2(
                        |errors| {
                            notif!(warning "Errors while importing", errors &errors);
                        },
                        || {
                            notif!(success "Saved project");
                        },
                    );
                }
                Err(e) => {
                    notif!(error "Errors while importing", error &e);
                }
            }
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
