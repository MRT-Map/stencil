use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use egui_file_dialog::FileDialog;
use itertools::Itertools;

use crate::{
    action::Action,
    history::{HistoryAct, HistoryEntry, NamespaceAction},
    project::events::ProjectAct,
    ui::panel::dock::{DockWindow, PanelParams, TabViewer},
};

#[derive(Clone, Copy)]
pub struct ProjectEditor;

impl DockWindow for ProjectEditor {
    fn title(self) -> String {
        "Project".into()
    }
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui) {
        let PanelParams {
            namespaces,
            new_namespace,
            actions,
            queries,
            ..
        } = tab_viewer.params;
        let components = queries.p1().iter().counts_by(|a| a.namespace.to_owned());
        ui.horizontal(|ui| {
            if ui.button("Select project folder").clicked() {
                actions.send(Action::new(ProjectAct::SelectFolder));
            }
            if ui.button("Reload").clicked() {
                actions.send(Action::new(ProjectAct::GetNamespaces));
            }
            if ui.button("Save").clicked() {
                actions.send(Action::new(ProjectAct::Save(false)));
            }
        });
        ui.label(format!(
            "Project folder: {}",
            namespaces.folder.to_string_lossy()
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
                                    actions.send(Action::new(ProjectAct::Show {
                                        ns: ns.to_owned(),
                                        history_invoked: false,
                                        notif: true,
                                    }));
                                } else {
                                    actions.send(Action::new(ProjectAct::Hide {
                                        ns: ns.to_owned(),
                                        history_invoked: false,
                                        notif: true,
                                    }));
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
                    actions.send(Action::new(ProjectAct::Delete(delete, false)));
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

                            actions.send(Action::new(HistoryAct::one_history(
                                HistoryEntry::Namespace {
                                    namespace: new_namespace.to_owned(),
                                    action: NamespaceAction::Create(None),
                                },
                            )));
                            new_namespace.clear();
                        }
                    });
                    row.col(|_| ());
                });
            });
    }
    fn closeable(self) -> bool {
        false
    }
}

impl ProjectEditor {
    #[must_use]
    pub fn select_dialog() -> FileDialog {
        FileDialog::new().title("Select project folder")
    }
}
