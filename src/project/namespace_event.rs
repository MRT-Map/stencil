use std::fmt::{Display, Formatter};

use crate::{
    App, notif,
    project::history::Event,
    utils::{file::safe_delete, with_warnings::ErrorWarningsExt},
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NamespaceEv {
    Load(String),
    Hide(String),
    Create(String),
    Delete(String),
}

impl Event for NamespaceEv {
    #[tracing::instrument(skip_all, fields(self))]
    fn run(&self, app: &mut App) -> bool {
        match self {
            Self::Load(namespace) => {
                let Ok(()) = app.project.load_namespace(namespace).notify(
                    format!("Error while loading `{namespace}`"),
                    format!("Errors while loading `{namespace}`"),
                ) else {
                    return false;
                };
                notif!(success format!("Loaded namespace `{namespace}`"));
                app.project.namespaces.insert(namespace.clone(), true);
                true
            }
            Self::Hide(namespace) => {
                let components = app
                    .project
                    .components
                    .iter()
                    .filter(|a| a.full_id.namespace == *namespace);
                let ww = app.project.save_components(components);
                let has_errors = !ww.warnings.is_empty();
                ww.notify(format!("Errors while saving `{namespace}`"));
                if has_errors {
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
                    notif!(warning format!("Error while creating `{namespace}`"), error &e);
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
                    let _ = safe_delete(path.join(namespace));
                }
                app.project.components.remove_namespace(namespace);
                app.project.namespaces.remove(namespace);
                notif!(success format!("Deleted namespace `{namespace}`"));
                true
            }
        }
    }
    fn undo(&self, app: &mut App) -> bool {
        match self {
            Self::Load(ns) => Self::Hide(ns.clone()),
            Self::Hide(ns) => Self::Load(ns.clone()),
            Self::Create(ns) => Self::Delete(ns.clone()),
            Self::Delete(ns) => Self::Create(ns.clone()),
        }
        .run(app)
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
