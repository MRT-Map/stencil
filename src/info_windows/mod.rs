pub mod changelog;
// pub mod info;
// pub mod licenses;
// pub mod manual;
// pub mod quit;

use std::collections::VecDeque;

use crate::{
    App,
    event::{Event, Events},
    info_windows::changelog::ChangelogPopup,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InfoWindowEv {
    Changelog,
    Info,
    Licenses,
    Manual,
    Quit { confirm: bool },
}

impl Event for InfoWindowEv {
    fn react(self, app: &mut App) {
        match self {
            Self::Changelog => app.add_popup(ChangelogPopup),
            _ => {}
        }
    }
}
