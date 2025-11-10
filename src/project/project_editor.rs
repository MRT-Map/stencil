use std::fmt::{Display, Formatter};

use egui_notify::ToastLevel;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    App,
    file::safe_delete,
    project::{Project, SkinStatus, event::Event},
    settings::settings_ui_field,
    shortcut::{ShortcutAction, UiButtonWithShortcutExt},
    ui::dock::DockWindow,
};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ProjectEditorWindow {
    new_namespace: String,
}

impl DockWindow for ProjectEditorWindow {
    fn title(&self) -> String {
        "Project Editor".into()
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            if ui
                .button_with_shortcut(
                    "Open",
                    ShortcutAction::OpenProject,
                    &mut app.shortcut_settings,
                )
                .clicked()
            {
                // commands.trigger(ProjectEv::Open);
            }
            if ui
                .button_with_shortcut(
                    "Reload",
                    ShortcutAction::ReloadProject,
                    &mut app.shortcut_settings,
                )
                .clicked()
            {
                // commands.trigger(ProjectEv::Reload);
            }
            if ui
                .button_with_shortcut(
                    "Save",
                    ShortcutAction::SaveProject,
                    &mut app.shortcut_settings,
                )
                .clicked()
            {
                app.project.save_notif(&mut app.ui.notifs);
            }
        });
        ui.separator();

        ui.label(format!(
            "Project directory: {}",
            app.project
                .path
                .as_ref()
                .map_or_else(|| "SCRATCHPAD".into(), |a| a.to_string_lossy())
        ));
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .column(egui_extras::Column::auto().at_least(0.05))
            .column(egui_extras::Column::remainder())
            .column(egui_extras::Column::auto().at_least(0.05))
            .column(egui_extras::Column::auto().at_least(0.05))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("👁").on_hover_text("Visibility");
                });
                header.col(|ui| {
                    ui.label("Namespace");
                });
                header.col(|ui| {
                    ui.label("#");
                });
                header.col(|ui| {
                    ui.label(" ");
                });
            })
            .body(|mut body| {
                for (ns, mut vis) in app.project.namespaces.clone() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            if app.project.path.is_none() {
                                return;
                            }
                            if ui.checkbox(&mut vis, "").changed() {
                                if vis {
                                    app.run_event(ProjectEv::Load(ns.clone()), ui.ctx());
                                } else {
                                    app.run_event(ProjectEv::Hide(ns.clone()), ui.ctx());
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.label(egui::RichText::new(&ns).code());
                        });
                        let num_components = app.project.namespace_component_count(&ns);
                        row.col(|ui| {
                            ui.label(
                                num_components
                                    .as_ref()
                                    .map_or_else(|_| "-".into(), |a| format!("{a}")),
                            );
                        });
                        row.col(|ui| {
                            if ui
                                .add_enabled(
                                    matches!(num_components, Ok(0)),
                                    egui::Button::new("❌").fill(egui::Color32::DARK_RED),
                                )
                                .clicked()
                            {
                                app.run_event(ProjectEv::Delete(ns), ui.ctx());
                            }
                        });
                    });
                }

                body.row(20.0, |mut row| {
                    row.col(|_| ());
                    row.col(|ui| {
                        egui::TextEdit::singleline(&mut self.new_namespace)
                            .hint_text("New namespace")
                            .show(ui);
                    });
                    row.col(|ui| {
                        if ui
                            .add_enabled(
                                !self.new_namespace.is_empty()
                                    && !app.project.namespaces.contains_key(&self.new_namespace),
                                egui::Button::new("➕"),
                            )
                            .clicked()
                        {
                            app.run_event(
                                ProjectEv::Create(std::mem::take(&mut self.new_namespace)),
                                ui.ctx(),
                            );
                        }
                    });
                });
            });

        ui.separator();

        match &app.project.skin_status {
            SkinStatus::Unloaded => {
                ui.colored_label(egui::Color32::ORANGE, "Skin is unloaded");
            }
            SkinStatus::Loading(_) => {
                ui.colored_label(egui::Color32::YELLOW, "Skin is loading");
            }
            SkinStatus::Failed(e) => {
                ui.colored_label(egui::Color32::RED, "Skin failed to load");
                ui.code(format!("{e:?}"));
            }
            SkinStatus::Loaded(_) => {
                ui.colored_label(egui::Color32::GREEN, "Skin is loaded");
            }
        }

        ui.separator();

        ui.heading("Configuration");
        ui.collapsing("Skin", |ui| {
            settings_ui_field(
                ui,
                &mut app.project.skin_url,
                Project::default().skin_url,
                Option::<&str>::None,
                |ui, value| {
                    ui.add(egui::TextEdit::singleline(value).desired_width(200.0));
                    // ui.text_edit_singleline(value);
                    ui.label("Skin URL");
                    if ui.button("Reload").clicked() {
                        app.project.skin_status = SkinStatus::Unloaded;
                    }
                },
            );
        });
        ui.collapsing("Basemap", |ui| {
            app.project.basemap.config_ui(ui);
        });
    }
}

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
