use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use tracing::debug;

use crate::{
    App, info_windows::InfoWindowEv, project::project_editor::ProjectEv, ui::dock::ResetLayoutEv,
};

#[enum_dispatch]
pub trait Event: Debug + Sized {
    fn log_react(self, ctx: &egui::Context, app: &mut App) {
        debug!(?self, "Reacting to event");
        self.react(ctx, app);
    }
    fn react(self, ctx: &egui::Context, app: &mut App);
}

#[expect(clippy::enum_variant_names)]
#[enum_dispatch(Event)]
#[derive(Clone, Debug)]
pub enum Events {
    InfoWindowEv,
    ResetLayoutEv,
    ProjectEv,
}

impl App {
    pub fn push_event<E: Into<Events>>(&mut self, event: E) {
        self.events.push_back(event.into());
    }
}
