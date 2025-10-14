use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ui::{
    dock::DockLayout,
    popup::{Popup, Popups},
};

pub mod dock;
pub mod menu_bar;
pub mod popup;

#[derive(Deserialize, Serialize, Default)]
pub struct UiState {
    #[serde(skip)]
    status: egui::RichText,
    dock_layout: DockLayout,
    popups: HashMap<String, Popups>,
}
