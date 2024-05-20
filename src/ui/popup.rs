use std::{
    any::Any,
    collections::HashMap,
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{misc::Action, ui::UiSet};

#[derive(Event, Hash, PartialEq, Eq, Clone)]
pub struct Popup(Arc<PopupInner<dyn Any + Send + Sync>>);

impl Deref for Popup {
    type Target = Arc<PopupInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Popup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct PopupInner<T: Send + Sync + ?Sized = dyn Any + Send + Sync> {
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

impl<T: Send + Sync + ?Sized> Hash for PopupInner<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: Send + Sync + ?Sized> PartialEq<Self> for PopupInner<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Send + Sync + ?Sized> Eq for PopupInner<T> {}

impl Popup {
    pub fn new<
        I: Display,
        W: Fn() -> egui::Window<'static> + Sync + Send + 'static,
        U: Fn(
                &Mutex<Box<dyn Any + Send + Sync>>,
                &mut egui::Ui,
                &mut EventWriter<Action>,
                &mut bool,
            ) + Sync
            + Send
            + 'static,
    >(
        id: I,
        window: W,
        ui: U,
        state: Mutex<Box<dyn Any + Send + Sync>>,
    ) -> Self {
        Self(Arc::new(PopupInner {
            id: id.to_string(),
            window: Box::new(window),
            ui: Box::new(ui),
            state,
        }))
    }
    pub fn base_alert<
        I: Display + Send + Sync + 'static,
        T1: Into<egui::WidgetText> + Clone + Sync + Send + 'static,
        T2: Into<egui::WidgetText> + Clone + Sync + Send + 'static,
    >(
        id: I,
        title: T1,
        text: T2,
    ) -> Self {
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
            move |_, ui, _, shown| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(text.to_owned());
                });
                if ui.button("Close").clicked() {
                    *shown = false;
                }
            },
            Mutex::new(Box::new(())),
        )
    }
    pub fn base_confirm<
        I: Display + Send + Sync + 'static,
        T1: Into<egui::WidgetText> + Clone + Sync + Send + 'static,
        T2: Into<egui::WidgetText> + Clone + Sync + Send + 'static,
    >(
        id: I,
        title: T1,
        text: T2,
        action: Action,
    ) -> Self {
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
            move |_, ui, ew, shown| {
                ui.label(text.to_owned());
                if ui.button("Yes").clicked() {
                    ew.send(action.to_owned());
                    *shown = false;
                }
                if ui.button("No").clicked() {
                    *shown = false;
                }
            },
            Mutex::new(Box::new(())),
        )
    }
}

#[tracing::instrument(skip_all)]
pub fn popup_handler_sy(
    mut ctx: EguiContexts,
    mut event_reader: EventReader<Popup>,
    mut event_writer: EventWriter<Action>,
    mut show: Local<HashMap<String, (Popup, bool)>>,
) {
    for popup in event_reader.read() {
        info!(popup.id, "Showing popup");
        show.insert(popup.id.to_owned(), (Popup::clone(popup), true));
    }
    let Some(ctx) = ctx.try_ctx_mut() else {
        return;
    };
    for (id, (popup, shown)) in &mut show {
        (popup.window)()
            .show(ctx, |ui| {
                (popup.ui)(&popup.state, ui, &mut event_writer, shown);
            })
            .unwrap();
        if !*shown {
            info!(?id, "Closing popup");
        }
    }
    show.retain(|_, (_, a)| *a);
}

pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Popup>()
            .add_systems(Update, popup_handler_sy.in_set(UiSet::Popups));
    }
}
