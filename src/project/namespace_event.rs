use std::fmt::{Display, Formatter};

use itertools::Itertools;

use crate::{App, file::safe_delete, notif, project::history::Event};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NamespaceEv {
    Load(String),
    Hide(String),
    Create(String),
    Delete(String),
}

impl Event for NamespaceEv {
    #[tracing::instrument(skip_all, fields(self))]
    fn run(&self, _ctx: &egui::Context, app: &mut App) -> bool {
        match self {
            Self::Load(namespace) => match app.project.load_namespace(namespace) {
                Ok(errors) => {
                    if !errors.is_empty() {
                        notif!(warning format!("Errors while loading `{namespace}`"), errors &errors);
                    }
                    notif!(success format!("Loaded namespace `{namespace}`"));
                    app.project.namespaces.insert(namespace.clone(), true);
                    true
                }
                Err(e) => {
                    let errors = [e];
                    notif!(error format!("Error while loading `{namespace}`"), errors &errors);
                    false
                }
            },
            Self::Hide(namespace) => {
                let components = app
                    .project
                    .components
                    .iter()
                    .filter(|a| a.full_id.namespace == *namespace);
                let errors = app.project.save_components(components, &mut app.ui.notifs);
                if !errors.is_empty() {
                    notif!(warning format!("Errors while saving `{namespace}`"), errors &errors);
                    return false;
                }
                app.project.components.remove_namespace(namespace);
                notif!(success format!("Hid namespace `{namespace}`"));
                app.project.namespaces.insert(namespace.clone(), false);
                true
            }
            Self::Create(namespace) => {
                if let Some(path) = &app.project.path
                    && let Err(e) = std::fs::create_dir_all(path.join(namespace))
                {
                    let errors = [e];
                    notif!(warning format!("Error while creating `{namespace}`"), errors &errors);
                }
                notif!(success format!("Created namespace `{namespace}`"));
                app.project.namespaces.insert(namespace.clone(), true);
                app.project.new_component_ns.clone_from(namespace);
                true
            }
            Self::Delete(namespace) => {
                if app
                    .project
                    .components
                    .iter()
                    .any(|a| a.full_id.namespace == *namespace)
                {
                    notif!(error format!("Attempted to delete non-empty namespace `{namespace}`"));
                    return false;
                }
                if let Some(path) = &app.project.path {
                    let _ = safe_delete(path.join(namespace), &mut app.ui.notifs);
                }
                app.project.components.remove_namespace(namespace);
                app.project.namespaces.remove(namespace);
                notif!(success format!("Deleted namespace `{namespace}`"));
                true
            }
        }
    }
    fn undo(&self, ctx: &egui::Context, app: &mut App) -> bool {
        match self {
            Self::Load(ns) => Self::Hide(ns.clone()),
            Self::Hide(ns) => Self::Load(ns.clone()),
            Self::Create(ns) => Self::Delete(ns.clone()),
            Self::Delete(ns) => Self::Create(ns.clone()),
        }
        .run(ctx, app)
    }
}

impl Display for NamespaceEv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(ns) => write!(f, "Load namespace {ns}"),
            Self::Hide(ns) => write!(f, "Hide namespace {ns}"),
            Self::Create(ns) => write!(f, "Create namespace {ns}"),
            Self::Delete(ns) => write!(f, "Delete namespace {ns}"),
        }
    }
}
