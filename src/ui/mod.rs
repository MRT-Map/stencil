use std::collections::HashMap;

use crate::ui::{dock::DockLayout, notif::NotifState, popup::Popups};

pub mod dock;
pub mod menu_bar;
pub mod notif;
pub mod popup;

pub struct UiState {
    pub status: egui::RichText,
    pub dock_layout: DockLayout,
    pub popups: HashMap<String, Popups>,
    pub notifs: NotifState,
    pub mspf: egui::util::History<f32>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            status: Default::default(),
            dock_layout: Default::default(),
            popups: Default::default(),
            notifs: Default::default(),
            mspf: egui::util::History::new(1..usize::MAX, 1.0),
        }
    }
}
