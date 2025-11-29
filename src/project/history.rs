use std::{
    collections::VecDeque,
    fmt::{Debug, Display, Formatter},
};

use enum_dispatch::enum_dispatch;
use tracing::debug;

use crate::{App, component_actions::event::ComponentEv, project::event::ProjectEv};

#[enum_dispatch]
pub trait Event: Debug + Sized + Display {
    fn run(&self, ctx: &egui::Context, app: &mut App) -> bool;
    fn undo(&self, ctx: &egui::Context, app: &mut App) -> bool;
}

#[enum_dispatch(Event)]
#[derive(Clone, Debug)]
pub enum Events {
    ProjectEv,
    ComponentEv,
}

impl Display for Events {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::ProjectEv(e) => write!(f, "{e}"),
            Self::ComponentEv(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Default, Debug)]
pub struct History {
    pub undo_stack: VecDeque<Events>,
    pub redo_stack: VecDeque<Events>,
}

impl History {
    pub fn add_event<E: Into<Events>>(&mut self, event: E) {
        let event = event.into();
        if let Events::ComponentEv(ComponentEv::ChangeField {
            before: before2,
            after: after2,
            label: label2,
        }) = &event
        {
            if before2 == after2 {
                return;
            }
            if !label2.is_empty()
                && !["move", "nodes"].contains(&&**label2)
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
        } else {
            self.undo_stack.push_back(event);
            self.redo_stack.clear();
        }
    }
}

impl App {
    pub fn run_event<E: Into<Events>>(&mut self, event: E, ctx: &egui::Context) {
        let event = event.into();
        debug!(?event, "Running event");
        if event.run(ctx, self) {
            self.project.history.add_event(event);
        }
    }
    pub fn add_event<E: Into<Events>>(&mut self, event: E) {
        self.project.history.add_event(event);
    }
    pub fn history_undo(&mut self, ctx: &egui::Context) {
        let Some(event) = self.project.history.undo_stack.pop_back() else {
            return;
        };
        debug!(?event, "Undoing event");
        if event.undo(ctx, self) {
            self.status_undo(&event, ctx);
            self.project.history.redo_stack.push_front(event);
        } else {
            self.project.history.undo_stack.push_back(event);
        }
    }
    pub fn history_redo(&mut self, ctx: &egui::Context) {
        let Some(event) = self.project.history.redo_stack.pop_front() else {
            return;
        };
        debug!(?event, "Redoing event");
        if event.run(ctx, self) {
            self.status_redo(&event, ctx);
            self.project.history.undo_stack.push_back(event);
        } else {
            self.project.history.redo_stack.push_front(event);
        }
    }
}
