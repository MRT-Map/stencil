use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
    misc::Action,
    ui::{HoveringOverGui, UiStage},
};

#[allow(clippy::type_complexity)]
pub struct Popup {
    pub id: &'static str,
    pub window: Box<dyn Fn() -> egui::Window<'static> + Sync + Send>,
    pub ui: Box<dyn Fn(&mut egui::Ui, &mut EventWriter<Action>, &mut bool) + Sync + Send>,
}

impl Hash for Popup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq<Self> for Popup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Popup {}

pub fn popup_handler(
    mut ctx: ResMut<EguiContext>,
    mut event_reader: EventReader<Arc<Popup>>,
    mut event_writer: EventWriter<Action>,
    mut show: Local<HashMap<Arc<Popup>, bool>>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
) {
    for popup in event_reader.iter() {
        show.insert(popup.to_owned(), true);
    }
    let ctx = ctx.ctx_mut();
    for (popup, showed) in show.iter_mut() {
        let response: egui::InnerResponse<Option<()>> = (popup.window)()
            .show(ctx, |ui| (popup.ui)(ui, &mut event_writer, showed))
            .unwrap();
        if response.response.hovered() {
            hovering_over_gui.0 = true;
        }
    }
    show.retain(|_, a| *a);
}

pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Arc<Popup>>()
            .add_system_to_stage(UiStage, popup_handler);
    }
}
