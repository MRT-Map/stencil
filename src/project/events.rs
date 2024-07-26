use std::path::PathBuf;

use bevy::{
    hierarchy::DespawnRecursiveExt,
    prelude::{Commands, Entity, Event, EventWriter, NonSendMut, Query, Res, ResMut, Trigger},
};
use bevy_egui::EguiContexts;
use egui_notify::ToastLevel;
use itertools::Itertools;

use crate::{
    component::{
        bundle::{AreaComponentBundle, LineComponentBundle, PointComponentBundle},
        pla2::{ComponentType, EditorCoords, MCCoords, PlaComponent},
        skin::Skin,
    },
    file::{load_msgpack, safe_delete, save_msgpack},
    history::{History, HistoryEntry, HistoryEv, NamespaceAction},
    project::Namespaces,
    ui::{
        notif::{NotifLogRwLockExt, NOTIF_LOG},
        panel::dock::FileDialogs,
        popup::Popup,
    },
};

#[derive(Clone, Event)]
pub enum ProjectEv {
    Open,
    Load(PathBuf, bool),
    Reload,
    Show {
        ns: String,
        history_invoked: bool,
        notif: bool,
    },
    Hide {
        ns: String,
        history_invoked: bool,
        notif: bool,
    },
    Delete(String, bool),
    Save(bool),
}

#[allow(clippy::needless_pass_by_value, clippy::cognitive_complexity)]
pub fn on_project(
    trigger: Trigger<ProjectEv>,
    mut commands: Commands,
    mut namespaces: ResMut<Namespaces>,
    query: Query<(Entity, &PlaComponent<EditorCoords>)>,
    mut file_dialogs: NonSendMut<FileDialogs>,
    skin: Res<Skin>,
    mut popup: EventWriter<Popup>,
    mut history: ResMut<History>,
) {
    match trigger.event() {
        ProjectEv::Show {
            ns,
            history_invoked,
            notif,
        } => {
            if !namespaces.dir.join(format!("{ns}.pla2.msgpack")).exists() {
                return;
            }
            namespaces.visibilities.insert(ns.to_owned(), true);
            if let Ok(components) = load_msgpack::<Vec<PlaComponent<MCCoords>>>(
                &namespaces.dir.join(format!("{ns}.pla2.msgpack")),
                Some("pla2"),
            ) {
                for component in components {
                    match component.get_type(&skin) {
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
                if !history_invoked {
                    commands.trigger(HistoryEv::one_history(HistoryEntry::Namespace {
                        namespace: ns.to_owned(),
                        action: NamespaceAction::Show,
                    }));
                }
                if *notif {
                    NOTIF_LOG.push(&format!("Loaded namespace {ns}"), ToastLevel::Success);
                }
            }
        }
        ProjectEv::Hide {
            ns,
            history_invoked,
            notif,
        } => {
            namespaces.visibilities.insert(ns.to_owned(), false);
            let components = query
                .iter()
                .filter(|(_, p)| p.namespace == *ns)
                .collect::<Vec<_>>();
            let component_data = components
                .iter()
                .map(|(_, p)| p.to_mc_coords())
                .collect::<Vec<_>>();
            if save_msgpack(
                &component_data
                    .iter()
                    .sorted_by_key(|a| &a.id)
                    .collect::<Vec<_>>(),
                &namespaces.dir.join(format!("{ns}.pla2.msgpack")),
                Some("pla2"),
            )
            .is_err()
            {
                return;
            }
            for (e, _) in components {
                commands.entity(e).despawn_recursive();
            }
            if !history_invoked {
                commands.trigger(HistoryEv::one_history(HistoryEntry::Namespace {
                    namespace: ns.to_owned(),
                    action: NamespaceAction::Hide,
                }));
            }
            if *notif {
                NOTIF_LOG.push(&format!("Saved namespace {ns}"), ToastLevel::Success);
            }
        }
        ProjectEv::Save(auto) => {
            let components = query
                .iter()
                .map(|(_, p)| p.to_mc_coords())
                .into_group_map_by(|a| a.namespace.to_owned());
            for (ns, components) in &components {
                let _ = save_msgpack(
                    &components
                        .iter()
                        .sorted_by_key(|a| &a.id)
                        .collect::<Vec<_>>(),
                    &namespaces.dir.join(format!("{ns}.pla2.msgpack")),
                    Some("pla2"),
                );
            }
            NOTIF_LOG.push(
                &if *auto {
                    format!("Auto-saved {} namespaces", components.len())
                } else {
                    format!("Saved {} namespaces", components.len())
                },
                ToastLevel::Success,
            );
        }
        ProjectEv::Open => {
            file_dialogs.project_select.select_directory();
        }
        ProjectEv::Reload => {
            let ns: Vec<String> = namespaces
                .dir
                .read_dir()
                .and_then(|rd| {
                    rd.into_iter()
                        .filter_map_ok(|rd| {
                            rd.path()
                                .file_name()
                                .map(|a| a.to_string_lossy().to_string())
                        })
                        .filter_map_ok(|p| p.strip_suffix(".pla2.msgpack").map(ToOwned::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            NOTIF_LOG.push(
                &format!("Reloaded {} namespaces", ns.len()),
                ToastLevel::Success,
            );
            for ns in ns {
                let _ = namespaces.visibilities.entry(ns).or_insert(false);
            }
        }
        ProjectEv::Delete(ns, false) => {
            popup.send(Popup::base_confirm(
                "confirm_delete_ns",
                format!("Are you sure you want to delete namespace {ns}?"),
                "",
                ProjectEv::Delete(ns.to_owned(), true),
            ));
        }
        ProjectEv::Delete(ns, true) => {
            namespaces.visibilities.remove(ns);
            let delete_file = namespaces
                .dir
                .join(format!("{ns}.pla2.msgpack"))
                .exists()
                .then(|| {
                    safe_delete(
                        &namespaces.dir.join(format!("{ns}.pla2.msgpack")),
                        Some("namespace file"),
                    )
                    .ok()
                })
                .flatten();
            commands.trigger(HistoryEv::one_history(HistoryEntry::Namespace {
                namespace: ns.to_owned(),
                action: NamespaceAction::Delete(delete_file),
            }));
        }
        ProjectEv::Load(dir, true) => {
            commands.trigger(ProjectEv::Save(false));
            commands.trigger(ProjectEv::Load(dir.to_owned(), false));
        }
        ProjectEv::Load(dir, false) => {
            history.redo_stack.clear();
            history.undo_stack.clear();
            namespaces.dir = dir.to_owned();
            namespaces.visibilities.clear();
            for (e, _) in query.iter() {
                commands.entity(e).despawn_recursive();
            }
            commands.trigger(ProjectEv::Reload);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn project_dialog(
    mut commands: Commands,
    namespaces: Res<Namespaces>,
    mut ctx: EguiContexts,
    mut file_dialogs: NonSendMut<FileDialogs>,
    mut popup: EventWriter<Popup>,
) {
    let file_dialog = &mut file_dialogs.project_select;
    let Some(ctx) = ctx.try_ctx_mut() else { return };
    file_dialog.update(ctx);
    if let Some(file) = file_dialog.take_selected() {
        if namespaces.dir == Namespaces::default().dir {
            commands.trigger(ProjectEv::Load(file, true));
        } else {
            popup.send(Popup::base_choose(
                "save-before-switching",
                "Save before switching projects?",
                "",
                ProjectEv::Load(file.to_owned(), true),
                ProjectEv::Load(file, false),
            ));
        }
    }
}
