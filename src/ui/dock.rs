use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    App,
    file::data_path,
    impl_load_save,
    map::MapWindow,
    project::{
        component_editor::ComponentEditorWindow, history_viewer::HistoryViewerWindow,
        project_editor::ProjectEditorWindow,
    },
    settings::SettingsWindow,
    ui::notif::NotifLogWindow,
};

#[enum_dispatch]
pub trait DockWindow: Copy {
    fn title(self) -> String;
    fn allowed_in_windows(self) -> bool {
        true
    }
    fn is_closeable(self) -> bool {
        true
    }
    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui);
}

#[expect(clippy::enum_variant_names)]
#[enum_dispatch(DockWindow)]
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(tag = "ty")]
pub enum DockWindows {
    MapWindow,
    ComponentEditorWindow,
    ProjectEditorWindow,
    SettingsWindow,
    NotifLogWindow,
    // ComponentList,
    HistoryViewerWindow,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DockLayout(pub egui_dock::DockState<DockWindows>);

impl_load_save!(json DockLayout, data_path("dock.json"));

impl Default for DockLayout {
    fn default() -> Self {
        let mut state = egui_dock::DockState::new(vec![MapWindow.into()]);
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
                ProjectEditorWindow.into(),
                // ComponentList.into(),
                HistoryViewerWindow.into(),
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

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
}

impl App {
    pub fn dock(&mut self, ui: &mut egui::Ui) {
        let mut dock_state = self.ui.dock_layout.0.clone();
        egui_dock::DockArea::new(&mut dock_state)
            .style(egui_dock::Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, self);
        self.ui.dock_layout.0 = dock_state;
    }
}
impl DockLayout {
    pub fn open_window<W: Into<DockWindows>>(&mut self, window: W) {
        let window = window.into();
        let tab_path = self.0.find_tab_from(|a| a.title() == window.title());
        if let Some(tab_path) = tab_path {
            info!("Focusing on {}", window.title());
            let _ = self
                .0
                .set_active_tab(tab_path)
                .inspect_err(|e| error!("{e:#}"));
        } else {
            info!("Creating new window {}", window.title());
            self.0.add_window(vec![window]);
        }
    }
}
