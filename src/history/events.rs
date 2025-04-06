use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use bevy::{hierarchy::DespawnRecursiveExt, prelude::*};
use tracing::debug;

use crate::{
    component::{
        bundle::{
            AreaComponentBundle, LineComponentBundle,
            PointComponentBundle,
        },
        pla2::ComponentType,
        skin::Skin,
    },
    file::{restore, safe_delete},
    history::{History, HistoryEntry, HistoryEv, NamespaceAction},
    project::{events::ProjectEv, Namespaces},
    ui::panel::status::Status,
};
use crate::component::actions::rendering::RenderEv;

#[expect(
    clippy::needless_pass_by_value,
    clippy::cognitive_complexity,
    clippy::significant_drop_tightening
)]
pub fn on_history(
    trigger: Trigger<HistoryEv>,
    mut commands: Commands,
    mut ids: Local<HashMap<Entity, Arc<RwLock<Entity>>>>,
    mut history: ResMut<History>,
    skin: Res<Skin>,
    mut status: ResMut<Status>,
    mut namespaces: ResMut<Namespaces>,
) {
    match trigger.event() {
        HistoryEv::NewHistory(histories) => {
            let histories = histories
                .iter()
                .map(|history| match history {
                    HistoryEntry::Component {
                        before,
                        after,
                        e: component_id,
                    } => HistoryEntry::Component {
                        before: before.to_owned(),
                        after: after.to_owned(),
                        e: {
                            let component_id = Arc::clone(
                                ids.entry(*component_id)
                                    .or_insert_with(|| Arc::new(RwLock::new(*component_id))),
                            );
                            debug!(?component_id, "Added entry to undo stack");
                            component_id
                        },
                    },
                    HistoryEntry::Namespace { namespace, action } => HistoryEntry::Namespace {
                        namespace: namespace.to_owned(),
                        action: action.to_owned(),
                    },
                })
                .collect::<Vec<_>>();
            history.redo_stack.clear();
            if let (
                Some(
                    [HistoryEntry::Component {
                        e: e1,
                        after: a1,
                        ..
                    }],
                ),
                [HistoryEntry::Component {
                    e: e2,
                    after: a2,
                    ..
                }],
            ) = (
                history.undo_stack.last_mut().map(Vec::as_mut_slice),
                histories.as_slice(),
            ) {
                if *e1.read().unwrap() == *e2.read().unwrap() {
                    a2.clone_into(a1);
                    return;
                }
            }
            history.undo_stack.push(histories);
        }
        HistoryEv::Undo => {
            let Some(mut histories) = history.undo_stack.pop() else {
                status.0 = "Nothing to undo".into();
                return;
            };
            for history in &mut histories {
                debug!("Undid {history}");
                status.0 = format!("Undid {history}").into();
                match history {
                    HistoryEntry::Component {
                        before,
                        after,
                        e: component_id,
                    } => match (before, after) {
                        (Some(before), None) => {
                            let e = match before.get_type(&skin) {
                                ComponentType::Point => commands
                                    .spawn(PointComponentBundle::new((**before).clone(), &skin)),
                                ComponentType::Line => commands
                                    .spawn(LineComponentBundle::new((**before).clone(), &skin)),
                                ComponentType::Area => commands
                                    .spawn(AreaComponentBundle::new((**before).clone(), &skin)),
                            }
                            .id();
                            *component_id.write().unwrap() = e;
                            ids.insert(e, Arc::clone(component_id));
                        }
                        (Some(before), Some(_)) => {
                            let component_id = component_id.read().unwrap();
                            commands.entity(*component_id).insert((**before).clone()).trigger(RenderEv::default());
                        }
                        (None, _) => {
                            let component_id = component_id.read().unwrap();
                            commands.entity(*component_id).despawn_recursive();
                            ids.remove(&component_id);
                        }
                    },
                    HistoryEntry::Namespace { namespace, action } => {
                        commands.trigger(match action {
                            NamespaceAction::Show => ProjectEv::Hide {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                                notif: true,
                            },
                            NamespaceAction::Hide => ProjectEv::Show {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                                notif: true,
                            },
                            NamespaceAction::Create(deleted_file) => {
                                namespaces.visibilities.remove(namespace);
                                if namespaces
                                    .dir
                                    .join(format!("{namespace}.pla2.msgpack"))
                                    .exists()
                                {
                                    *deleted_file = safe_delete(
                                        &namespaces.dir.join(format!("{namespace}.pla2.msgpack")),
                                        Some("namespace file"),
                                    )
                                    .ok();
                                }
                                continue;
                            }
                            NamespaceAction::Delete(deleted_file) => {
                                namespaces.visibilities.insert(namespace.to_owned(), false);
                                if let Some(deleted_file) = deleted_file {
                                    let _ = restore(
                                        deleted_file,
                                        &namespaces.dir.join(format!("{namespace}.pla2.msgpack")),
                                        Some("namespace file"),
                                    )
                                    .ok();
                                }
                                continue;
                            }
                        });
                    }
                }
            }
            history.redo_stack.push(histories);
        }
        HistoryEv::Redo => {
            let Some(mut histories) = history.redo_stack.pop() else {
                status.0 = "Nothing to redo".into();
                return;
            };
            for history in &mut histories {
                debug!("Redid {history}");
                status.0 = format!("Redid {history}").into();
                match history {
                    HistoryEntry::Component {
                        before,
                        after,
                        e: component_id,
                    } => match (before, after) {
                        (None, Some(after)) => {
                            let e = match after.get_type(&skin) {
                                ComponentType::Point => commands
                                    .spawn(PointComponentBundle::new((**after).clone(), &skin)),
                                ComponentType::Line => commands
                                    .spawn(LineComponentBundle::new((**after).clone(), &skin)),
                                ComponentType::Area => commands
                                    .spawn(AreaComponentBundle::new((**after).clone(), &skin)),
                            }
                            .id();
                            *component_id.write().unwrap() = e;
                            ids.insert(e, Arc::clone(component_id));
                        }
                        (Some(_), Some(after)) => {
                            let component_id = component_id.read().unwrap();
                            commands.entity(*component_id).insert((**after).clone()).trigger(RenderEv::default());
                        }
                        (_, None) => {
                            let component_id = component_id.read().unwrap();
                            commands.entity(*component_id).despawn_recursive();
                            ids.remove(&component_id);
                        }
                    },
                    HistoryEntry::Namespace { namespace, action } => {
                        commands.trigger(match action {
                            NamespaceAction::Show => ProjectEv::Show {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                                notif: true,
                            },
                            NamespaceAction::Hide => ProjectEv::Hide {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                                notif: true,
                            },
                            NamespaceAction::Create(deleted_file) => {
                                namespaces.visibilities.insert(namespace.to_owned(), true);
                                if let Some(deleted_file) = deleted_file {
                                    let _ = restore(
                                        deleted_file,
                                        &namespaces.dir.join(format!("{namespace}.pla2.msgpack")),
                                        Some("namespace file"),
                                    )
                                    .ok();
                                }
                                continue;
                            }
                            NamespaceAction::Delete(deleted_file) => {
                                namespaces.visibilities.remove(namespace);
                                if namespaces
                                    .dir
                                    .join(format!("{namespace}.pla2.msgpack"))
                                    .exists()
                                {
                                    *deleted_file = safe_delete(
                                        &namespaces.dir.join(format!("{namespace}.pla2.msgpack")),
                                        Some("namespace file"),
                                    )
                                    .ok();
                                }
                                continue;
                            }
                        });
                    }
                }
            }
            history.undo_stack.push(histories);
        }
    }
}
