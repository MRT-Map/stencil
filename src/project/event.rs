use std::{collections::VecDeque, fmt::Debug};

use enum_dispatch::enum_dispatch;
use tracing::debug;

use crate::{App, component_actions::ComponentEv, project::project_editor::ProjectEv};

#[enum_dispatch]
pub trait Event: Debug + Sized {
    fn run(&self, ctx: &egui::Context, app: &mut App) -> bool;
    fn undo(&self, ctx: &egui::Context, app: &mut App) -> bool;
}

#[enum_dispatch(Event)]
#[derive(Clone, Debug)]
pub enum Events {
    ProjectEv,
    ComponentEv,
}

impl App {
    pub fn run_event<E: Into<Events>>(&mut self, event: E, ctx: &egui::Context) {
        let event = event.into();
        debug!(?event, "Running event");
        if event.run(ctx, self) {
            self.project.undo_tree.add_event(event);
        }
    }
    pub fn add_event<E: Into<Events>>(&mut self, event: E) {
        self.project.undo_tree.add_event(event);
    }
}

#[derive(Default, Debug)]
pub struct UndoTree {
    undo_stack: VecDeque<Events>,
    redo_stack: VecDeque<Events>,
}

impl UndoTree {
    pub fn add_event<E: Into<Events>>(&mut self, event: E) {
        let event = event.into();
        if let Events::ComponentEv(ComponentEv::ChangeField {
            before: before2,
            after: after2,
            label: label2,
        }) = &event
            && !label2.is_empty()
            && let Some(Events::ComponentEv(ComponentEv::ChangeField {
                before: before1,
                after: after1,
                label: label1,
            })) = self.undo_stack.back_mut()
            && label2 == label1
            && after1 == before2
        {
            if before1 == after2 {
                self.undo_stack.pop_back();
            } else {
                after1.clone_from(after2);
            }
        } else {
            self.undo_stack.push_back(event);
            self.redo_stack.clear();
        }
    }
    pub fn undo(&mut self, ctx: &egui::Context, app: &mut App) {
        let Some(event) = self.undo_stack.pop_back() else {
            return;
        };
        debug!(?event, "Undoing event");
        if event.undo(ctx, app) {
            self.redo_stack.push_front(event);
        } else {
            self.undo_stack.push_back(event);
        }
    }
    pub fn redo(&mut self, ctx: &egui::Context, app: &mut App) {
        let Some(event) = self.redo_stack.pop_front() else {
            return;
        };
        debug!(?event, "Redoing event");
        if event.run(ctx, app) {
            self.undo_stack.push_back(event);
        } else {
            self.redo_stack.push_front(event);
        }
    }
}
