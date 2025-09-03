use std::{any::Any, fmt::Display, sync::Mutex};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::ui::{EguiPrimaryContextPass, UiSet};

#[derive(Resource, Default)]
pub struct Popups(pub Vec<Popup>);

impl Popups {
    pub fn add(&mut self, popup: Popup) {
        if self.0.iter().any(|a| a.id == popup.id) {
            return;
        }
        info!(?popup.id, "Opening popup");
        self.0.push(popup);
    }
}

pub struct Popup<T: Send + Sync + ?Sized = dyn Any + Send + Sync> {
    pub id: String,
    pub window: Box<dyn Fn() -> egui::Window<'static> + Sync + Send>,
    pub ui: Box<
        dyn Fn(&Mutex<Box<T>>, &mut egui::Ui, &mut Commands, &mut bool) + Sync + Send + 'static,
    >,
    pub state: Mutex<Box<T>>,
}

impl Popup {
    pub fn new<
        I: Display,
        W: Fn() -> egui::Window<'static> + Sync + Send + 'static,
        U: Fn(&Mutex<Box<dyn Any + Send + Sync>>, &mut egui::Ui, &mut Commands, &mut bool)
            + Sync
            + Send
            + 'static,
    >(
        id: I,
        window: W,
        ui: U,
        state: Mutex<Box<dyn Any + Send + Sync>>,
    ) -> Self {
        Self {
            id: id.to_string(),
            window: Box::new(window),
            ui: Box::new(ui),
            state,
        }
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
        E: Event + Clone + Send + Sync + 'static,
    >(
        id: I,
        title: T1,
        text: T2,
        action: E,
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
            move |_, ui, commands, shown| {
                ui.label(text.to_owned());
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        commands.trigger(action.to_owned());
                        *shown = false;
                    }
                    if ui.button("No").clicked() {
                        *shown = false;
                    }
                });
            },
            Mutex::new(Box::new(())),
        )
    }
    pub fn base_choose<
        I: Display + Send + Sync + 'static,
        T1: Into<egui::WidgetText> + Clone + Sync + Send + 'static,
        T2: Into<egui::WidgetText> + Clone + Sync + Send + 'static,
        E1: Event + Clone + Send + Sync + 'static,
        E2: Event + Clone + Send + Sync + 'static,
    >(
        id: I,
        title: T1,
        text: T2,
        action1: E1,
        action2: E2,
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
            move |_, ui, commands, shown| {
                ui.label(text.to_owned());
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        commands.trigger(action1.to_owned());
                        *shown = false;
                    }
                    if ui.button("No").clicked() {
                        commands.trigger(action2.to_owned());
                        *shown = false;
                    }
                });
            },
            Mutex::new(Box::new(())),
        )
    }
}

#[tracing::instrument(skip_all)]
pub fn popup_handler_sy(mut ctx: EguiContexts, mut commands: Commands, mut popups: ResMut<Popups>) {
    let Ok(ctx) = ctx.ctx_mut() else {
        return;
    };

    popups.0.retain(|popup| {
        let mut shown = true;
        (popup.window)()
            .show(ctx, |ui| {
                (popup.ui)(&popup.state, ui, &mut commands, &mut shown);
            })
            .unwrap();
        if !shown {
            info!(?popup.id, "Closing popup");
        }
        shown
    });
}

pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Popups>().add_systems(
            EguiPrimaryContextPass,
            popup_handler_sy.in_set(UiSet::Popups),
        );
    }
}
