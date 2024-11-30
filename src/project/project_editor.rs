use bevy::prelude::{Event, ResMut, Trigger};
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use egui_file_dialog::FileDialog;
use itertools::Itertools;

use crate::{
    history::{HistoryEntry, HistoryEv, NamespaceAction},
    project::events::ProjectEv,
    ui::panel::dock::{window_action_handler, DockWindow, PanelDockState, PanelParams, TabViewer},
};

#[derive(Clone, Copy)]
pub struct ProjectEditor;

#[derive(Clone, Copy, Event)]
pub struct OpenProjectEditorEv;

impl DockWindow for ProjectEditor {
    fn title(self) -> String {
        "Project".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams {
            namespaces,
            new_namespace,
            commands,
            queries,
            ..
        } = tab_viewer.params;
        let components = queries.p1().iter().counts_by(|a| a.namespace.clone());
        ui.horizontal(|ui| {
            if ui.button("Open").clicked() {
                commands.trigger(ProjectEv::Open);
            }
            if ui.button("Reload").clicked() {
                commands.trigger(ProjectEv::Reload);
            }
            if ui.button("Save").clicked() {
                commands.trigger(ProjectEv::Save(false));
            }
        });
        ui.label(format!(
            "Project directory: {}",
            namespaces.dir.to_string_lossy()
        ));
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(0.05))
            .columns(Column::auto().at_least(25.0), 2)
            .column(Column::auto().at_least(0.05))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("Vis.");
                });
                header.col(|ui| {
                    ui.label("Ns.");
                });
                header.col(|ui| {
                    ui.label("#");
                });
                header.col(|ui| {
                    ui.label("Del.");
                });
            })
            .body(|mut body| {
                let mut delete = None;
                for (ns, vis) in &mut namespaces.visibilities {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            if ui.checkbox(vis, "").changed() {
                                if *vis {
                                    commands.trigger(ProjectEv::Show {
                                        ns: ns.to_owned(),
                                        history_invoked: false,
                                        notif: true,
                                    });
                                } else {
                                    commands.trigger(ProjectEv::Hide {
                                        ns: ns.to_owned(),
                                        history_invoked: false,
                                        notif: true,
                                    });
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.label(egui::RichText::new(ns).code());
                        });
                        let num_components = components.get(ns).copied().unwrap_or_default();
                        row.col(|ui| {
                            ui.label(if *vis {
                                num_components.to_string()
                            } else {
                                "-".into()
                            });
                        });
                        row.col(|ui| {
                            if ui
                                .add_enabled(
                                    num_components == 0 && ns != "_misc" && !*vis,
                                    egui::Button::new("X"),
                                )
                                .clicked()
                            {
                                delete = Some(ns.to_owned());
                            }
                        });
                    });
                }
                if let Some(delete) = delete {
                    commands.trigger(ProjectEv::Delete(delete, false));
                }
                body.row(20.0, |mut row| {
                    row.col(|_| ());
                    row.col(|ui| {
                        egui::TextEdit::singleline(&mut **new_namespace)
                            .hint_text("New ns.")
                            .show(ui);
                    });
                    row.col(|ui| {
                        if ui
                            .add_enabled(
                                !new_namespace.is_empty()
                                    && !namespaces.visibilities.keys().contains(&**new_namespace),
                                egui::Button::new("+"),
                            )
                            .clicked()
                        {
                            namespaces
                                .visibilities
                                .insert(new_namespace.to_owned(), true);

                            commands.trigger(HistoryEv::one_history(HistoryEntry::Namespace {
                                namespace: new_namespace.to_owned(),
                                action: NamespaceAction::Create(None),
                            }));
                            new_namespace.clear();
                        }
                    });
                    row.col(|_| ());
                });
            });
    }
}

impl ProjectEditor {
    #[must_use]
    pub fn select_dialog() -> FileDialog {
        FileDialog::new().title("Open project")
    }
}

pub fn on_project_editor(
    _trigger: Trigger<OpenProjectEditorEv>,
    mut state: ResMut<PanelDockState>,
) {
    window_action_handler(&mut state, ProjectEditor);
}
