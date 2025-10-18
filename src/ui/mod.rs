use std::collections::HashMap;

use crate::ui::{dock::DockLayout, notif::NotifState, popup::Popups};

pub mod dock;
pub mod menu_bar;
pub mod notif;
pub mod popup;

#[derive(Default)]
pub struct UiState {
    pub status: egui::RichText,
    pub dock_layout: DockLayout,
    pub popups: HashMap<String, Popups>,
    pub notifs: NotifState,
}
