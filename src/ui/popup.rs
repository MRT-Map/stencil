use std::{
    any::Any,
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_egui::{
    egui,
    egui::{Pos2, WidgetText},
    EguiContext,
};
use bevy_mouse_tracking_plugin::MousePos;

use crate::{
    misc::{Action, CustomStage},
    ui::HoveringOverGui,
};

#[allow(clippy::type_complexity)]
pub struct Popup<T: Send + Sync + ?Sized = dyn Any + Send + Sync> {
    pub id: String,
    pub window: Box<dyn Fn() -> egui::Window<'static> + Sync + Send>,
    pub ui: Box<
        dyn Fn(&Mutex<Box<T>>, &mut egui::Ui, &mut EventWriter<Action>, &mut bool)
            + Sync
            + Send
            + 'static,
    >,
    pub state: Mutex<Box<T>>,
}

impl<T: Send + Sync + ?Sized> Hash for Popup<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl<T: Send + Sync + ?Sized> PartialEq<Self> for Popup<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Send + Sync + ?Sized> Eq for Popup<T> {}

impl Popup {
    pub fn new(
        id: impl std::fmt::Display,
        window: impl Fn() -> egui::Window<'static> + Sync + Send + 'static,
        ui: impl Fn(
                &Mutex<Box<dyn Any + Send + Sync>>,
                &mut egui::Ui,
                &mut EventWriter<Action>,
                &mut bool,
            ) + Sync
            + Send
            + 'static,
        state: Mutex<Box<dyn Any + Send + Sync>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id: id.to_string(),
            window: Box::new(window),
            ui: Box::new(ui),
            state,
        })
    }
    pub fn base_alert(
        id: impl std::fmt::Display + Send + Sync + 'static,
        title: impl Into<WidgetText> + Clone + Sync + Send + 'static,
        text: impl Into<WidgetText> + Clone + Sync + Send + 'static,
    ) -> Arc<Self> {
        let win_id = egui::Id::new(id.to_string());
        Self::new(
            id.to_string(),
            move || {
                egui::Window::new(title.to_owned())
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .id(win_id)
            },
            move |_, ui, ew, show| {
                ui.label(text.to_owned());
                if ui.button("Close").clicked() {
                    ew.send(Action {
                        id: id.to_string(),
                        payload: Box::new(()),
                    });
                    *show = false;
                }
            },
            Mutex::new(Box::new(())),
        )
    }
    pub fn base_confirm(
        id: impl std::fmt::Display + Send + Sync + 'static,
        title: impl Into<WidgetText> + Clone + Sync + Send + 'static,
        text: impl Into<WidgetText> + Clone + Sync + Send + 'static,
        payload: impl Any + Sync + Send + Clone,
    ) -> Arc<Self> {
        let win_id = egui::Id::new(id.to_string());
        Self::new(
            id.to_string(),
            move || {
                egui::Window::new(title.to_owned())
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .id(win_id)
            },
            move |_, ui, ew, show| {
                ui.label(text.to_owned());
                if ui.button("Yes").clicked() {
                    ew.send(Action {
                        id: id.to_string(),
                        payload: Box::new(payload.to_owned()),
                    });
                    *show = false;
                }
                if ui.button("No").clicked() {
                    *show = false;
                }
            },
            Mutex::new(Box::new(())),
        )
    }
}

#[tracing::instrument(skip_all)]
pub fn popup_handler(
    mut ctx: ResMut<EguiContext>,
    mut event_reader: EventReader<Arc<Popup>>,
    mut event_writer: EventWriter<Action>,
    mut show: Local<HashMap<String, (Arc<Popup>, bool)>>,
    mut hovering_over_gui: ResMut<HoveringOverGui>,
    mouse_pos: Res<MousePos>,
) {
    for popup in event_reader.iter() {
        info!(popup.id, "Showing popup");
        show.insert(popup.id.to_owned(), (popup.to_owned(), true));
    }
    let ctx = ctx.ctx_mut();
    for (id, (popup, showed)) in show.iter_mut() {
        let response: egui::InnerResponse<Option<()>> = (popup.window)()
            .show(ctx, |ui| {
                (popup.ui)(&popup.state, ui, &mut event_writer, showed)
            })
            .unwrap();
        if response.response.hovered()
            || response
                .response
                .rect
                .contains(Pos2::from(mouse_pos.to_array()))
        {
            hovering_over_gui.0 = true;
        }
        if !*showed {
            info!(?id, "Closing popup")
        }
    }
    show.retain(|_, (_, a)| *a);
}

pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Arc<Popup>>()
            .add_system_to_stage(CustomStage::Ui, popup_handler.before("ui_menu"));
    }
}
