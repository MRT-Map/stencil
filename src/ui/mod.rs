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

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UiState {
    #[serde(skip)]
    status: egui::RichText,
    dock_layout: DockLayout,
    #[serde(skip)]
    popups: HashMap<String, Popups>,
    #[serde(skip)]
    notifs: NotifState,
}
