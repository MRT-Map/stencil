use std::collections::HashMap;

use crate::{
    map::state::MapState,
    ui::{dock::DockLayout, notif::NotifState, popup::Popups},
};

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
    pub map: MapState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            status: egui::RichText::default(),
            dock_layout: DockLayout::default(),
            popups: HashMap::default(),
            notifs: NotifState::default(),
            mspf: egui::util::History::new(1..usize::MAX, 1.0),
            map: MapState::default(),
        }
    }
}
