use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
    fs::File,
    io::{BufReader, Cursor, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};

use eyre::{Report, eyre};
use pla::{FullId, Pla2File};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    App,
    component_actions::{event::ComponentEv, paint::TOLERANCE},
    map::{basemap::Basemap, state::MapState},
    notif,
    project::{Project, history::Events, namespace_event::NamespaceEv, pla3::PlaComponent},
    ui::popup::Popup,
    utils::{
        coord::CoordFrom,
        file::safe_write,
        warnings::{ResultExt, ResultWithWarningsExt, WithWarning, WithWarnings},
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

    #[tracing::instrument(skip_all)]
    pub fn import_namespace_pla2(
        &self,
        path: &Path,
    ) -> eyre::Result<WithWarnings<(String, Vec<Events>)>> {
        let mut errors = Vec::new();
        let mut events = Vec::<Events>::new();
        let Some(format) = path
            .extension()
            .and_then(|a| a.to_str())
            .filter(|a| ["msgpack", "json"].contains(a))
        else {
            return Err(eyre!(
                "File `{}` must end with `.pla2.msgpack` or `.pla2.json`",
                path.display()
            ));
        };

        let pla2_file = if format == "msgpack" {
            Pla2File::<geo::Coord<i32>>::from_msgpack_bytes(&std::fs::read(path)?)?
        } else {
            Pla2File::<geo::Coord<i32>>::from_json_bytes(&std::fs::read(path)?)?
        };

        if pla2_file.namespace.is_empty() {
            return Err(eyre!("Namespace name must not be empty"));
        }
        if !self.namespaces.contains_key(&pla2_file.namespace) {
            events.push(NamespaceEv::Create(pla2_file.namespace.clone()).into());
        }

        let components = pla2_file
            .components
            .into_iter()
            .filter_map(|pla2| {
                if pla2.namespace == pla2_file.namespace {
                    pla2.to_pla3(|a| self.skin()?.get_type(a).map(Arc::clone))
                        .error_to_vec(&mut errors)
                } else {
                    errors.push(eyre!(
                        "Component `{}-{}` in file `{}` has invalid namespace",
                        pla2.namespace,
                        pla2.id,
                        path.display()
                    ));
                    None
                }
            })
            .collect::<Vec<_>>();

        events.push(ComponentEv::Create(components).into());

        Ok(WithWarnings::new((pla2_file.namespace, events), errors))
    }

    #[tracing::instrument(skip_all)]
    pub fn export_namespace_pla2(
        &self,
        namespace: &str,
        path: &Path,
        format: Pla2Format,
    ) -> eyre::Result<()> {
        let components = self
            .components
            .iter_namespace(namespace)
            .map(|a| a.clone().map_coords(egui::Pos2::coord_from))
            .map(|a| a.to_pla2(|ty| ty.name().clone(), TOLERANCE))
            .collect();
        let pla2_file = Pla2File {
            namespace: namespace.into(),
            components,
        };

        safe_write(
            path,
            match format {
                Pla2Format::MessagePack => pla2_file.to_msgpack_bytes()?,
                Pla2Format::Json => pla2_file.to_json_bytes()?,
            },
        )?;

        Ok(())
    }
}

impl App {
    #[tracing::instrument(skip_all)]
    pub fn open_project(&mut self) {
        let Some(folder) = FileDialog::new().set_title("Open Project").pick_folder() else {
            return;
        };
        let Ok(project) = Project::load(folder).notify("Failed to load project") else {
            return;
        };
        self.project = project;
        self.ui.map = MapState::default();
    }
    #[tracing::instrument(skip_all)]
    pub fn reload_project(&mut self) {
        if self.project.path.is_none() {
            return;
        }
        let Ok(()) = self
            .project
            .update_namespace_list()
            .notify_w("Error(s) while reloading project")
        else {
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
                .notify_w("Error(s) while importing")
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
        self.add_popup(ChooseNamespacesPopup::new(
            egui::WidgetText::default(),
            ChooseNamespacesPopupAction::ExportPla3,
        ));
    }
    #[tracing::instrument(skip_all)]
    pub fn export_namespace_pla3_zip(&self, namespace: &str) {
        let Some(file) = FileDialog::new()
            .set_title("Export pla3.zip")
            .set_file_name(format!("{namespace}.pla3.zip"))
            .add_filter("pla3.zip file", &["pla3.zip"])
            .save_file()
        else {
            return;
        };
        let Ok(()) = self
            .project
            .export_namespace_pla3_zip(namespace, &file)
            .notify("Error while exporting")
        else {
            return;
        };
        notif!(success format!("Exported to {}", file.display()));
    }

    #[tracing::instrument(skip_all)]
    pub fn import_namespace_pla2(&mut self) {
        let Some(files) = FileDialog::new()
            .set_title("Import pla2")
            .add_filter("pla2 file", &["pla2.json", "pla2.msgpack"])
            .pick_files()
        else {
            return;
        };
        for file in files {
            let Ok((namespace, events)) = self
                .project
                .import_namespace_pla2(&file)
                .notify_w("Error(s) while importing")
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
    pub fn export_namespaces_pla2(&mut self, format: Pla2Format) {
        self.add_popup(ChooseNamespacesPopup::new(
            egui::RichText::new("NOTE: PLA2 does not support Bézier curves. Any curves will be approximated and may not show up accurately in renders.").color(egui::Color32::YELLOW).into(),
            ChooseNamespacesPopupAction::ExportPla2(format),
        ));
    }
    #[tracing::instrument(skip_all)]
    pub fn export_namespace_pla2(&self, namespace: &str, format: Pla2Format) {
        let Some(file) = FileDialog::new()
            .set_title("Export pla2")
            .set_file_name(format!("{namespace}.pla2.{format}"))
            .add_filter("pla2 file", &[format!(".pla2.{format}")])
            .save_file()
        else {
            return;
        };
        let Ok(()) = self
            .project
            .export_namespace_pla2(namespace, &file, format)
            .notify("Error while exporting")
        else {
            return;
        };
        notif!(success format!("Exported to {}", file.display()));
    }
    #[tracing::instrument(skip_all)]
    pub fn autosave(&self, ctx: &egui::Context) {
        if self.project.path.is_none() || self.settings.misc.autosave_duration_mins == 0 {
            return;
        }
        let id = egui::Id::new("last-save");
        let Some(last_save) = ctx.data(|d| d.get_temp::<SystemTime>(id)) else {
            ctx.data_mut(|d| d.insert_temp(id, SystemTime::now()));
            return;
        };
        if !SystemTime::now()
            .duration_since(last_save)
            .is_ok_and(|d| d > Duration::from_mins(self.settings.misc.autosave_duration_mins))
        {
            return;
        }
        self.project
            .save()
            .notify("Errors while autosaving project");
        notif!(success "Autosaved project");
    }
}

#[derive(Clone, Copy)]
pub enum Pla2Format {
    MessagePack,
    Json,
}
impl Display for Pla2Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MessagePack => write!(f, "msgpack"),
            Self::Json => write!(f, "json"),
        }
    }
}
#[derive(Clone, Copy)]
pub enum ChooseNamespacesPopupAction {
    ExportPla3,
    ExportPla2(Pla2Format),
}

#[derive(Clone)]
pub struct ChooseNamespacesPopup {
    selected: HashSet<String>,
    description: egui::WidgetText,
    action: ChooseNamespacesPopupAction,
}

impl ChooseNamespacesPopup {
    pub fn new(description: egui::WidgetText, action: ChooseNamespacesPopupAction) -> Self {
        Self {
            selected: HashSet::new(),
            description,
            action,
        }
    }
}

impl Popup for ChooseNamespacesPopup {
    fn id(&self) -> Cow<'static, str> {
        "choose-namespaces".into()
    }

    fn title(&self) -> egui::WidgetText {
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
            match self.action {
                ChooseNamespacesPopupAction::ExportPla3 => app.export_namespace_pla3_zip(namespace),
                ChooseNamespacesPopupAction::ExportPla2(format) => {
                    app.export_namespace_pla2(namespace, format);
                }
            }
        }
        false
    }
}
