use std::sync::Arc;

use bevy::prelude::*;

use crate::{action, misc::Action, ui::popup::Popup};

pub fn info_msy(mut actions: EventReader<Action>, mut popup: EventWriter<Arc<Popup>>) {
    action!(actions; "info", (), |_| {
        popup.send(Arc::new(Popup::base_alert(
            "info",
            format!("Stencil v{}", env!("CARGO_PKG_VERSION")),
            "Made by __7d for the MRT Mapping Services\n\nLinks would appear here..."
        )));
    });
}
