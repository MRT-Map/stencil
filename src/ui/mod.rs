use std::collections::HashMap;

use egui_notify::Toasts;
use serde::{Deserialize, Serialize};

use crate::ui::{
    dock::DockLayout,
    notif::NotifState,
    popup::{Popup, Popups},
};

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
