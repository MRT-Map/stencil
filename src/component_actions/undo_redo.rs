use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    misc::{Action, EditorState},
    pla2::{
        bundle::{ComponentBundle, SelectedComponent},
        component::{EditorCoords, PlaComponent},
        skin::Skin,
    },
};

#[derive(Clone, Debug)]
pub struct History<T> {
    pub component_id: T,
    pub before: Option<PlaComponent<EditorCoords>>,
    pub after: Option<PlaComponent<EditorCoords>>,
}

pub enum UndoRedoAct {
    NewHistory(Vec<History<Entity>>),
    Undo,
    Redo,
}
impl UndoRedoAct {
    pub fn one_history(history: History<Entity>) -> Self {
        Self::NewHistory(vec![history])
    }
}

pub fn undo_redo_asy(
    mut commands: Commands,
    mut actions: EventReader<Action>,
    mut ids: Local<HashMap<Entity, Arc<RwLock<Entity>>>>,
    mut undo_stack: Local<Vec<Vec<History<Arc<RwLock<Entity>>>>>>,
    mut redo_stack: Local<Vec<Vec<History<Arc<RwLock<Entity>>>>>>,
    selected_entity: Query<Entity, With<SelectedComponent>>,
    skin: Res<Skin>,
) {
    let selected = selected_entity.get_single().ok();
    for event in actions.iter() {
        if let Some(UndoRedoAct::NewHistory(histories)) = event.downcast_ref() {
            let histories = histories
                .iter()
                .map(|history| {
                    let component_id = ids
                        .entry(history.component_id)
                        .or_insert_with(|| Arc::new(RwLock::new(history.component_id)))
                        .clone();
                    debug!(?history.component_id, "Added entry to undo stack");
                    History {
                        component_id,
                        before: history.before.to_owned(),
                        after: history.after.to_owned(),
                    }
                })
                .collect();
            redo_stack.clear();
            undo_stack.push(histories);
        } else if let Some(UndoRedoAct::Undo) = event.downcast_ref() {
            let Some(mut histories) = undo_stack.pop() else {
                continue
            };
            for history in &mut histories {
                if let Some(before) = &mut history.before {
                    if history.after.is_none() {
                        debug!(?history.component_id, "Undoing deletion");
                        let entity = commands
                            .spawn(ComponentBundle::new(before.to_owned()))
                            .insert(before.get_shape(&skin, false))
                            .id();
                        *history.component_id.write().unwrap() = entity;
                        ids.insert(entity, history.component_id.clone());
                    } else {
                        debug!(?history.component_id, "Undoing edit");
                        let entity = *history.component_id.read().unwrap();
                        commands
                            .entity(entity)
                            .insert(before.to_owned())
                            .insert(before.get_shape(&skin, Some(entity) == selected));
                    }
                } else {
                    debug!(?history.component_id, "Undoing creation");
                    let entity = *history.component_id.read().unwrap();
                    commands.entity(entity).despawn_recursive();
                    ids.remove(&entity);
                }
            }
            redo_stack.push(histories);
        } else if let Some(UndoRedoAct::Redo) = event.downcast_ref() {
            let Some(mut histories) = redo_stack.pop() else {
                continue
            };
            for history in &mut histories {
                if let Some(after) = &mut history.after {
                    debug!(?history.component_id, "Redoing creation");
                    if history.before.is_none() {
                        let entity = commands
                            .spawn(ComponentBundle::new(after.to_owned()))
                            .insert(after.get_shape(&skin, false))
                            .id();
                        *history.component_id.write().unwrap() = entity;
                        ids.insert(entity, history.component_id.clone());
                    } else {
                        debug!(?history.component_id, "Redoing edit");
                        let entity = *history.component_id.read().unwrap();
                        commands
                            .entity(entity)
                            .insert(after.to_owned())
                            .insert(after.get_shape(&skin, Some(entity) == selected));
                    }
                } else {
                    debug!(?history.component_id, "Redoing deletion");
                    let entity = *history.component_id.read().unwrap();
                    commands.entity(entity).despawn_recursive();
                    ids.remove(&entity);
                }
            }
            undo_stack.push(histories);
        }
    }
}

pub struct UndoRedoPlugin;
impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(EditorState::Loading)
                .with_system(undo_redo_asy)
                .into(),
        );
    }
}
