use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_extras::{Column, TableBuilder};
use egui_file_dialog::FileDialog;
use itertools::Itertools;

use crate::{
    component::{
        bundle::{
            AreaComponentBundle, LineComponentBundle, PointComponentBundle, SelectedComponent,
        },
        pla2::{ComponentType, EditorCoords, MCCoords, PlaComponent},
        skin::Skin,
    },
    misc::{load_msgpack, load_toml, save_msgpack, Action},
    project::Namespaces,
    tile::tile_coord::URL_REPLACER,
    ui::{
        panel::{
            dock::{DockWindow, FileDialogs, PanelDockState, PanelParams, TabViewer},
            status::Status,
        },
        popup::Popup,
        tilemap::{
            settings::{Basemap, TileSettings},
            settings_editor::TileSettingsEditor,
        },
    },
};

pub enum ProjectAct {
    SelectFolder,
    GetNamespaces,
    Show(String),
    Hide(String),
    Save,
}

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
            ..
        } = &mut tab_viewer.params;
        ui.horizontal(|ui| {
            if ui.button("Select project folder").clicked() {
                actions.send(Action::new(ProjectAct::SelectFolder));
            }
            if ui.button("Reload").clicked() {
                actions.send(Action::new(ProjectAct::GetNamespaces));
            }
        });
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(0.05))
            .columns(Column::remainder(), 2)
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
                                    actions.send(Action::new(ProjectAct::Show(ns.to_owned())));
                                } else {
                                    actions.send(Action::new(ProjectAct::Hide(ns.to_owned())));
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.label(egui::RichText::new(ns).code());
                        });
                        row.col(|ui| {
                            ui.label("0");
                        });
                        row.col(|ui| {
                            if ui.add_enabled(false, egui::Button::new("X")).clicked() {
                                delete = Some(ns.to_owned());
                            }
                        });
                    });
                }
                if let Some(delete) = delete {
                    namespaces.visibilities.remove(&delete);
                }
                body.row(20.0, |mut row| {
                    row.col(|_| ());
                    row.col(|ui| {
                        egui::TextEdit::singleline(&mut **new_namespace)
                            .hint_text("New ns.")
                            .show(ui);
                    });
                    row.col(|ui| {
                        if ui.button("+").clicked() {
                            namespaces
                                .visibilities
                                .insert(new_namespace.to_owned(), true);
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

#[allow(clippy::needless_pass_by_value)]
pub fn project_msy(
    mut actions: ParamSet<(EventReader<Action>, EventWriter<Action>)>,
    mut namespaces: ResMut<Namespaces>,
    mut commands: Commands,
    query: Query<(Entity, &PlaComponent<EditorCoords>)>,
    mut ctx: EguiContexts,
    mut file_dialogs: NonSendMut<FileDialogs>,
    mut popup: EventWriter<Popup>,
    mut status: ResMut<Status>,
    skin: Res<Skin>,
) {
    for event in actions.p0().read() {
        if let Some(ProjectAct::Show(ns)) = event.downcast_ref() {
            if let Some(components) = load_msgpack::<Vec<PlaComponent<MCCoords>>>(
                &namespaces.folder.join(format!("{ns}.pla2.msgpack")),
                Some((&mut popup, "pla2")),
            ) {
                for component in components {
                    match component.get_type(&skin).unwrap() {
                        ComponentType::Point => commands.spawn(PointComponentBundle::new(
                            component.to_editor_coords(),
                            &skin,
                        )),
                        ComponentType::Line => commands.spawn(LineComponentBundle::new(
                            component.to_editor_coords(),
                            &skin,
                        )),
                        ComponentType::Area => commands.spawn(AreaComponentBundle::new(
                            component.to_editor_coords(),
                            &skin,
                        )),
                    };
                }
                status.0 = format!("Loaded namespace {ns}").into();
            }
        } else if let Some(ProjectAct::Hide(ns)) = event.downcast_ref() {
            let components = query
                .iter()
                .filter_map(|(e, p)| {
                    if p.namespace == *ns {
                        None
                    } else {
                        commands.entity(e).despawn_recursive();
                        Some(p)
                    }
                })
                .collect::<Vec<_>>();
            if save_msgpack(
                &components,
                &namespaces.folder.join(format!("{ns}.pla2.msgpack")),
                Some((&mut popup, "pla2")),
            ) {
                status.0 = format!("Saved namespace {ns}").into();
            }
        } else if matches!(event.downcast_ref(), Some(ProjectAct::Save)) {
            let components = query
                .iter()
                .map(|(_, p)| p)
                .into_group_map_by(|a| &a.namespace);
            for (ns, components) in &components {
                let _ = save_msgpack(
                    &components,
                    &namespaces.folder.join(format!("{ns}.pla2.msgpack")),
                    Some((&mut popup, "pla2")),
                );
            }
            status.0 = format!("Saved {} namespaces", components.len()).into();
        } else if matches!(event.downcast_ref(), Some(ProjectAct::SelectFolder)) {
            file_dialogs.project_select.select_directory();
        } else if matches!(event.downcast_ref(), Some(ProjectAct::GetNamespaces)) {
            let ns: Vec<String> = namespaces
                .folder
                .read_dir()
                .and_then(|rd| {
                    rd.into_iter()
                        .filter_map_ok(|rd| {
                            rd.path()
                                .file_name()
                                .map(|a| a.to_string_lossy().to_string())
                        })
                        .filter_map_ok(|p| p.strip_suffix(".pla2.msgpack").map(|a| a.to_owned()))
                        .collect()
                })
                .unwrap_or_default();
            for ns in ns {
                let _ = namespaces.visibilities.entry(ns).or_insert(false);
            }
        }
    }

    let file_dialog = &mut file_dialogs.project_select;
    file_dialog.update(ctx.ctx_mut());
    if let Some(file) = file_dialog.take_selected() {
        namespaces.folder = file;
        namespaces.visibilities.clear();
        for (e, _) in query.iter() {
            commands.entity(e).despawn_recursive();
        }
        actions.p1().send(Action::new(ProjectAct::GetNamespaces));
    }
}
