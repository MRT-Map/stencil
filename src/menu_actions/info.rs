use std::sync::Arc;

use bevy::prelude::*;

use crate::{misc::Action, ui::popup::Popup};

pub fn info_msy(mut actions: EventReader<Action>, mut popup: EventWriter<Arc<Popup>>) {
    for event in actions.iter() {
        if event.id == "info" {
            popup.send(Arc::new(Popup::base_alert(
                "info",
                format!("Stencil v{}", env!("CARGO_PKG_VERSION")),
                "Made by __7d for the MRT Mapping Services\n\nLinks would appear here...",
            )));
        }
    }
}
