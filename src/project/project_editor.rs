use egui::TextBuffer;
use egui_notify::ToastLevel;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    App, event::Event, file::safe_delete, project::pla3::PlaComponent, ui::dock::DockWindow,
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
            if ui.button("Open").clicked() {
                // commands.trigger(ProjectEv::Open);
            }
            if ui.button("Reload").clicked() {
                // commands.trigger(ProjectEv::Reload);
            }
            if ui
                .add_enabled(
                    app.project.path.is_some(),
                    egui::Button::new("Save").shortcut_text("TODO"),
                )
                .clicked()
            {
                app.events.push_back(ProjectEv::Save.into());
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
                for (ns, vis) in &app.project.namespaces {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            if app.project.path.is_none() {
                                return;
                            }
                            let mut new_vis = *vis;
                            if ui.checkbox(&mut new_vis, "").changed() {
                                if new_vis {
                                    app.events.push_back(ProjectEv::Load(ns.clone()).into());
                                } else {
                                    app.events.push_back(ProjectEv::Hide(ns.clone()).into());
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.label(egui::RichText::new(ns).code());
                        });
                        let num_components = app.project.namespace_component_count(ns);
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
                                app.events
                                    .push_back(ProjectEv::Delete(ns.to_owned()).into());
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
                            app.events
                                .push_back(ProjectEv::Create(self.new_namespace.take()).into());
                        }
                    });
                });
            });
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProjectEv {
    Load(String),
    Hide(String),
    Create(String),
    Delete(String),
    Save,
}

impl Event for ProjectEv {
    fn react(self, ctx: &egui::Context, app: &mut App) {
        match self {
            Self::Load(namespace) => match app.project.load_namespace(&namespace) {
                Ok(errors) => {
                    if !errors.is_empty() {
                        app.ui.notifs.push(
                            format!(
                                "Errors while loading `{namespace}`:\n{}",
                                errors.iter().map(|e| format!("{e}")).join("\n")
                            ),
                            ToastLevel::Warning,
                            &app.misc_settings,
                        );
                    }
                    app.ui.notifs.push(
                        format!("Loaded namespace `{namespace}`"),
                        ToastLevel::Success,
                        &app.misc_settings,
                    );
                    app.project.namespaces.insert(namespace, true);
                }
                Err(e) => app.ui.notifs.push(
                    format!("Error while loading `{namespace}`: {e}"),
                    ToastLevel::Error,
                    &app.misc_settings,
                ),
            },
            Self::Hide(namespace) => {
                let components = app
                    .project
                    .components
                    .iter()
                    .filter(|a| a.namespace == namespace)
                    .collect::<Vec<_>>();
                let errors = app.project.save_components(components);
                if !errors.is_empty() {
                    app.ui.notifs.push(
                        format!(
                            "Errors while saving `{namespace}`:\n{}",
                            errors.iter().map(|e| format!("{e}")).join("\n")
                        ),
                        ToastLevel::Warning,
                        &app.misc_settings,
                    );
                    return;
                }
                app.project.components.retain(|a| a.namespace != namespace);
                app.ui.notifs.push(
                    format!("Hid namespace `{namespace}`"),
                    ToastLevel::Success,
                    &app.misc_settings,
                );
                app.project.namespaces.insert(namespace, false);
            }
            Self::Create(namespace) => {
                if let Some(path) = &app.project.path
                    && let Err(e) = std::fs::create_dir_all(path.join(&namespace))
                {
                    app.ui.notifs.push(
                        format!("Error while creating `{namespace}`:\n{e}"),
                        ToastLevel::Warning,
                        &app.misc_settings,
                    );
                }
                app.ui.notifs.push(
                    format!("Created namespace `{namespace}`"),
                    ToastLevel::Success,
                    &app.misc_settings,
                );
                app.project.namespaces.insert(namespace, true);
            }
            Self::Delete(namespace) => {
                if app
                    .project
                    .components
                    .iter()
                    .any(|a| a.namespace == namespace)
                {
                    app.ui.notifs.push(
                        format!("Attempted to delete non-empty namespace `{namespace}`"),
                        ToastLevel::Error,
                        &app.misc_settings,
                    );
                    return;
                }
                if let Some(path) = &app.project.path {
                    let _ = safe_delete(
                        path.join(&namespace),
                        &app.misc_settings,
                        &mut app.ui.notifs,
                    );
                }
                app.project.components.retain(|a| a.namespace != namespace);
                app.project.namespaces.remove(&namespace);
                app.ui.notifs.push(
                    format!("Deleted namespace `{namespace}`"),
                    ToastLevel::Success,
                    &app.misc_settings,
                );
            }
            Self::Save => {
                if app.project.path.is_none() {
                    return;
                }
                let errors = app.project.save();
                if !errors.is_empty() {
                    app.ui.notifs.push(
                        format!(
                            "Errors while saving:\n{}",
                            errors.iter().map(|e| format!("{e}")).join("\n")
                        ),
                        ToastLevel::Warning,
                        &app.misc_settings,
                    );
                    return;
                }
                app.ui
                    .notifs
                    .push("Saved project", ToastLevel::Success, &app.misc_settings);
            }
        }
    }
}
