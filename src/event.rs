use std::{collections::VecDeque, fmt::Debug};

use enum_dispatch::enum_dispatch;
use tracing::debug;

use crate::{App, info_windows::InfoWindowEv};

#[enum_dispatch(Events)]
pub trait Event: Debug + Sized {
    fn log_react(self, app: &mut App) {
        debug!(?self, "Reacting to event");
        self.react(app);
    }
    fn react(self, app: &mut App);
}

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum Events {
    InfoWindowEv,
}
