use std::fmt::{Display, Formatter};

use egui_notify::ToastLevel;
use itertools::Itertools;

use crate::{App, file::safe_delete, project::history::Event};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProjectEv {
    Load(String),
    Hide(String),
    Create(String),
    Delete(String),
}

impl Event for ProjectEv {
    fn run(&self, _ctx: &egui::Context, app: &mut App) -> bool {
        match self {
            Self::Load(namespace) => match app.project.load_namespace(namespace) {
                Ok(errors) => {
                    if !errors.is_empty() {
                        app.ui.notifs.push(
                            format!(
                                "Errors while loading `{namespace}`:\n{}",
                                errors.iter().map(|e| format!("{e}")).join("\n")
                            ),
                            ToastLevel::Warning,
                        );
                    }
                    app.ui.notifs.push(
                        format!("Loaded namespace `{namespace}`"),
                        ToastLevel::Success,
                    );
                    app.project.namespaces.insert(namespace.clone(), true);
                    true
                }
                Err(e) => {
                    app.ui.notifs.push(
                        format!("Error while loading `{namespace}`: {e}"),
                        ToastLevel::Error,
                    );
                    false
                }
            },
            Self::Hide(namespace) => {
                let components = app
                    .project
                    .components
                    .iter()
                    .filter(|a| a.full_id.namespace == *namespace);
                let errors = app.project.save_components(components);
                if !errors.is_empty() {
                    app.ui.notifs.push_errors(
                        format!("Errors while saving `{namespace}`"),
                        &errors,
                        ToastLevel::Warning,
                    );
                    return false;
                }
                app.project.components.remove_namespace(namespace);
                app.ui
                    .notifs
                    .push(format!("Hid namespace `{namespace}`"), ToastLevel::Success);
                app.project.namespaces.insert(namespace.clone(), false);
                true
            }
            Self::Create(namespace) => {
                if let Some(path) = &app.project.path
                    && let Err(e) = std::fs::create_dir_all(path.join(namespace))
                {
                    app.ui.notifs.push_error(
                        format!("Error while creating `{namespace}`"),
                        e,
                        ToastLevel::Warning,
                    );
                }
                app.ui.notifs.push(
                    format!("Created namespace `{namespace}`"),
                    ToastLevel::Success,
                );
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
                    app.ui.notifs.push(
                        format!("Attempted to delete non-empty namespace `{namespace}`"),
                        ToastLevel::Error,
                    );
                    return false;
                }
                if let Some(path) = &app.project.path {
                    let _ = safe_delete(path.join(namespace), &mut app.ui.notifs);
                }
                app.project.components.remove_namespace(namespace);
                app.project.namespaces.remove(namespace);
                app.ui.notifs.push(
                    format!("Deleted namespace `{namespace}`"),
                    ToastLevel::Success,
                );
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

impl Display for ProjectEv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(ns) => write!(f, "Load namespace {ns}"),
            Self::Hide(ns) => write!(f, "Hide namespace {ns}"),
            Self::Create(ns) => write!(f, "Create namespace {ns}"),
            Self::Delete(ns) => write!(f, "Delete namespace {ns}"),
        }
    }
}
