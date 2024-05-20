use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use bevy::prelude::*;

use crate::{
    action::Action,
    component::{
        bundle::{
            AreaComponentBundle, EntityCommandsSelectExt, LineComponentBundle,
            PointComponentBundle, SelectedComponent,
        },
        pla2::{ComponentType, EditorCoords, PlaComponent},
        skin::Skin,
    },
    project::events::ProjectAct,
    state::IntoSystemConfigExt,
    ui::panel::status::Status,
};

#[derive(Clone, Debug)]
pub enum History<T = Entity> {
    Component {
        entity: T,
        before: Option<Box<PlaComponent<EditorCoords>>>,
        after: Option<Box<PlaComponent<EditorCoords>>>,
    },
    Namespace {
        namespace: String,
        visible: bool,
    },
}

pub enum UndoRedoAct {
    NewHistory(Vec<History>),
    Undo,
    Redo,
}
impl UndoRedoAct {
    #[must_use]
    pub fn one_history(history: History) -> Self {
        Self::NewHistory(vec![history])
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::cognitive_complexity,
    clippy::implicit_hasher
)]
pub fn undo_redo_asy(
    mut commands: Commands,
    mut actions: ParamSet<(EventReader<Action>, EventWriter<Action>)>,
    mut ids: Local<HashMap<Entity, Arc<RwLock<Entity>>>>,
    mut undo_stack: Local<Vec<Vec<History<Arc<RwLock<Entity>>>>>>,
    mut redo_stack: Local<Vec<Vec<History<Arc<RwLock<Entity>>>>>>,
    selected_entity: Query<Entity, With<SelectedComponent>>,
    skin: Res<Skin>,
    mut status: ResMut<Status>,
) {
    let selected = selected_entity.get_single().ok();
    let mut send_queue: Vec<Action> = vec![];
    for event in actions.p0().read() {
        if let Some(UndoRedoAct::NewHistory(histories)) = event.downcast_ref() {
            let histories = histories
                .iter()
                .map(|history| match history {
                    History::Component {
                        before,
                        after,
                        entity: component_id,
                    } => History::Component {
                        before: before.to_owned(),
                        after: after.to_owned(),
                        entity: {
                            let component_id = Arc::clone(
                                ids.entry(*component_id)
                                    .or_insert_with(|| Arc::new(RwLock::new(*component_id))),
                            );
                            debug!(?component_id, "Added entry to undo stack");
                            component_id
                        },
                    },
                    History::Namespace { namespace, visible } => History::Namespace {
                        namespace: namespace.to_owned(),
                        visible: visible.to_owned(),
                    },
                })
                .collect();
            redo_stack.clear();
            undo_stack.push(histories);
        } else if matches!(event.downcast_ref(), Some(UndoRedoAct::Undo)) {
            let Some(mut histories) = undo_stack.pop() else {
                continue;
            };
            for history in &mut histories {
                match history {
                    History::Component {
                        before,
                        after,
                        entity: component_id,
                    } => {
                        if let Some(before) = before {
                            if after.is_none() {
                                debug!(?component_id, "Undoing deletion");
                                status.0 = format!("Undid deletion of {before}").into();
                                let entity = match before.get_type(&skin) {
                                    ComponentType::Point => commands.spawn(
                                        PointComponentBundle::new((**before).to_owned(), &skin),
                                    ),
                                    ComponentType::Line => commands.spawn(
                                        LineComponentBundle::new((**before).to_owned(), &skin),
                                    ),
                                    ComponentType::Area => commands.spawn(
                                        AreaComponentBundle::new((**before).to_owned(), &skin),
                                    ),
                                }
                                .id();
                                *component_id.write().unwrap() = entity;
                                ids.insert(entity, Arc::clone(component_id));
                            } else {
                                let component_id = component_id.read().unwrap();
                                debug!(?component_id, "Undoing edit");
                                status.0 = format!("Undid edit of {before}").into();
                                commands.entity(*component_id).insert((**before).to_owned());
                                if Some(*component_id) == selected {
                                    commands
                                        .entity(*component_id)
                                        .select_component(&skin, before);
                                } else {
                                    commands
                                        .entity(*component_id)
                                        .component_display(&skin, before);
                                }
                            }
                        } else {
                            let component_id = component_id.read().unwrap();
                            debug!(?component_id, "Undoing creation");
                            status.0 = format!(
                                "Undid creation of {}",
                                after.as_ref().map_or_else(String::new, |a| format!("{a}"))
                            )
                            .into();
                            commands.entity(*component_id).despawn_recursive();
                            ids.remove(&component_id);
                        }
                    }
                    History::Namespace { namespace, visible } => {
                        send_queue.push(if *visible {
                            Action::new(ProjectAct::Hide {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                            })
                        } else {
                            Action::new(ProjectAct::Show {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                            })
                        });
                    }
                }
            }
            redo_stack.push(histories);
        } else if matches!(event.downcast_ref(), Some(UndoRedoAct::Redo)) {
            let Some(mut histories) = redo_stack.pop() else {
                continue;
            };
            for history in &mut histories {
                match history {
                    History::Component {
                        before,
                        after,
                        entity: component_id,
                    } => {
                        if let Some(after) = after {
                            debug!(?component_id, "Redoing creation");
                            status.0 = format!("Redid creation of {after}").into();
                            if before.is_none() {
                                let entity = match after.get_type(&skin) {
                                    ComponentType::Point => commands.spawn(
                                        PointComponentBundle::new((**after).to_owned(), &skin),
                                    ),
                                    ComponentType::Line => commands.spawn(
                                        LineComponentBundle::new((**after).to_owned(), &skin),
                                    ),
                                    ComponentType::Area => commands.spawn(
                                        AreaComponentBundle::new((**after).to_owned(), &skin),
                                    ),
                                }
                                .id();
                                *component_id.write().unwrap() = entity;
                                ids.insert(entity, Arc::clone(component_id));
                            } else {
                                let component_id = component_id.read().unwrap();
                                debug!(?component_id, "Redoing edit");
                                status.0 = format!("Redid edit of {after}").into();
                                commands.entity(*component_id).insert((**after).to_owned());
                                if Some(*component_id) == selected {
                                    commands
                                        .entity(*component_id)
                                        .select_component(&skin, after);
                                } else {
                                    commands
                                        .entity(*component_id)
                                        .component_display(&skin, after);
                                }
                            }
                        } else {
                            let component_id = component_id.read().unwrap();
                            debug!(?component_id, "Redoing deletion");
                            status.0 = format!(
                                "Redid deletion of {}",
                                before.as_ref().map_or_else(String::new, |a| format!("{a}"))
                            )
                            .into();
                            commands.entity(*component_id).despawn_recursive();
                            ids.remove(&component_id);
                        }
                    }
                    History::Namespace { namespace, visible } => {
                        send_queue.push(if *visible {
                            Action::new(ProjectAct::Show {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                            })
                        } else {
                            Action::new(ProjectAct::Hide {
                                ns: namespace.to_owned(),
                                history_invoked: true,
                            })
                        });
                    }
                }
            }
            undo_stack.push(histories);
        }
    }

    for action in send_queue {
        actions.p1().send(action);
    }
}

pub struct UndoRedoPlugin;
impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, undo_redo_asy.run_if_not_loading());
    }
}
