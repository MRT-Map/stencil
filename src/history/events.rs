use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use bevy::prelude::*;
use eyre::eyre;
use itertools::Itertools;
use tracing::debug;

use crate::{
    component::{actions::rendering::RenderEv, make_component, skin::Skin},
    file::{restore, safe_delete},
    history::{History, HistoryEntry, HistoryEv, NamespaceAction},
    project::{events::ProjectEv, Namespaces},
    ui::panel::status::Status,
};

#[expect(clippy::needless_pass_by_value, clippy::significant_drop_tightening)]
pub fn on_history(
    trigger: Trigger<HistoryEv>,
    mut commands: Commands,
    mut ids: Local<HashMap<Entity, Arc<RwLock<Entity>>>>,
    mut history: ResMut<History>,
    skin: Res<Skin>,
    mut status: ResMut<Status>,
    mut namespaces: ResMut<Namespaces>,
) -> Result {
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
                        e: e1, after: a1, ..
                    }],
                ),
                [HistoryEntry::Component {
                    e: e2, after: a2, ..
                }],
            ) = (
                history.undo_stack.last_mut().map(Vec::as_mut_slice),
                histories.as_slice(),
            ) {
                if *e1.read().map_err(|a| eyre!("{a:?}"))?
                    == *e2.read().map_err(|a| eyre!("{a:?}"))?
                {
                    a2.clone_into(a1);
                    return Ok(());
                }
            }
            history.undo_stack.push(histories);
        }
        HistoryEv::Undo | HistoryEv::Redo => {
            let history = &mut *history;
            let (ev, past, stack, other_stack) = if matches!(trigger.event(), HistoryEv::Undo) {
                (
                    "undo",
                    "Undid",
                    &mut history.undo_stack,
                    &mut history.redo_stack,
                )
            } else {
                (
                    "redo",
                    "Redid",
                    &mut history.redo_stack,
                    &mut history.undo_stack,
                )
            };
            let Some(mut histories) = stack.pop() else {
                status.set(format!("Nothing to {ev}"));
                return Ok(());
            };
            status.set(format!(
                "{past} {}",
                histories.iter().map(ToString::to_string).join("; ")
            ));
            for history in &mut histories {
                debug!("{past} {history}");
                match history {
                    HistoryEntry::Component {
                        before,
                        after,
                        e: component_id,
                    } => match (trigger.event(), before, after) {
                        (HistoryEv::Undo, Some(pla), None) | (HistoryEv::Redo, None, Some(pla)) => {
                            let e = commands.spawn(make_component((**pla).clone(), &skin)).id();
                            *component_id.write().map_err(|a| eyre!("{a:?}"))? = e;
                            ids.insert(e, Arc::clone(component_id));
                        }
                        (HistoryEv::Undo, Some(pla), Some(_))
                        | (HistoryEv::Redo, Some(_), Some(pla)) => {
                            let component_id = component_id.read().map_err(|a| eyre!("{a:?}"))?;
                            commands
                                .entity(*component_id)
                                .insert((**pla).clone())
                                .trigger(RenderEv::default());
                        }
                        (HistoryEv::Undo, None, _) | (HistoryEv::Redo, _, None) => {
                            let component_id = component_id.read().map_err(|a| eyre!("{a:?}"))?;
                            commands.entity(*component_id).despawn();
                            ids.remove(&component_id);
                        }
                        _ => unreachable!(),
                    },
                    HistoryEntry::Namespace { namespace, action } => {
                        commands.trigger(match (trigger.event(), action) {
                            (HistoryEv::Undo, NamespaceAction::Show)
                            | (HistoryEv::Redo, NamespaceAction::Hide) => ProjectEv::Hide {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                                notif: true,
                            },
                            (HistoryEv::Undo, NamespaceAction::Hide)
                            | (HistoryEv::Redo, NamespaceAction::Show) => ProjectEv::Show {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                                notif: true,
                            },
                            (HistoryEv::Undo, NamespaceAction::Create(deleted_file))
                            | (HistoryEv::Redo, NamespaceAction::Delete(deleted_file)) => {
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
                            (HistoryEv::Undo, NamespaceAction::Delete(deleted_file))
                            | (HistoryEv::Redo, NamespaceAction::Create(deleted_file)) => {
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
                            _ => unreachable!(),
                        });
                    }
                }
            }
            other_stack.push(histories);
        }
    }
    Ok(())
}
