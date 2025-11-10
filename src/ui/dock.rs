use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    App,
    file::data_path,
    impl_load_save,
    map::MapWindow,
    project::{
        component_editor::ComponentEditorWindow, event::Event, history_viewer::HistoryViewer,
        project_editor::ProjectEditorWindow,
    },
    settings::SettingsWindow,
    ui::notif::NotifLogWindow,
};

#[enum_dispatch]
pub trait DockWindow {
    fn title(&self) -> String;
    fn allowed_in_windows(&self) -> bool {
        true
    }
    fn is_closeable(&self) -> bool {
        true
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui);
}

#[expect(clippy::enum_variant_names)]
#[enum_dispatch(DockWindow)]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "ty")]
pub enum DockWindows {
    MapWindow,
    ComponentEditorWindow,
    ProjectEditorWindow,
    SettingsWindow,
    NotifLogWindow,
    // ComponentList,
    HistoryViewer,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DockLayout(pub egui_dock::DockState<DockWindows>);

impl_load_save!(mpk DockLayout, data_path("dock.mpk"));

impl Default for DockLayout {
    fn default() -> Self {
        let mut state = egui_dock::DockState::new(vec![MapWindow::default().into()]);
        let tree = state.main_surface_mut();
        let [_, _] = tree.split_left(
            egui_dock::NodeIndex::root(),
            0.2,
            vec![ComponentEditorWindow.into()],
        );
        let [_, _] = tree.split_right(
            egui_dock::NodeIndex::root(),
            0.8,
            vec![
                ProjectEditorWindow::default().into(),
                // ComponentList.into(),
                HistoryViewer.into(),
            ],
        );
        Self(state)
    }
}
impl DockLayout {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl egui_dock::TabViewer for App {
    type Tab = DockWindows;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(self, ui);
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        tab.is_closeable()
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        tab.allowed_in_windows()
    }
}

impl App {
    pub fn dock(&mut self, ctx: &egui::Context) {
        let mut dock_state = self.ui.dock_layout.0.clone();
        egui_dock::DockArea::new(&mut dock_state)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, self);
        self.ui.dock_layout.0 = dock_state;
    }
    pub fn open_dock_window<W: Into<DockWindows>>(&mut self, window: W) {
        let window = window.into();
        let a = self
            .ui
            .dock_layout
            .0
            .iter_all_tabs()
            .find(|(_, a)| a.title() == window.title());
        if let Some((a, _)) = a {
            info!("Focusing on {}", window.title());
            let a = a.to_owned();
            self.ui.dock_layout.0.set_focused_node_and_surface(a);
        } else {
            info!("Creating new window {}", window.title());
            self.ui.dock_layout.0.add_window(vec![window]);
        }
    }
}
