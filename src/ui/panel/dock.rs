use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{EguiContexts, egui};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabBodyStyle, TabStyle};
use egui_notify::ToastLevel;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::{
    component::{
        actions::selecting::SelectedComponent,
        panels::{component_editor::ComponentEditor, component_list::ComponentList},
        pla2::PlaComponent,
        skin::Skin,
    },
    dirs_paths::data_path,
    file::{load_msgpack, save_msgpack},
    history::{History, history_viewer::HistoryViewer},
    keymaps::{settings::KeymapSettings, settings_editor::KeymapSettingsEditor},
    misc_config::{settings::MiscSettings, settings_editor::MiscSettingsEditor},
    project::{Namespaces, project_editor::ProjectEditor},
    state::EditorState,
    ui::{
        cursor::mouse_pos::MousePosWorld,
        map::{
            settings::TileSettings, settings_editor::TileSettingsEditor, tiles::PendingTiles,
            window::Tilemap, zoom::Zoom,
        },
        notif::{NOTIF_LOG, NotifLogRwLockExt, viewer::NotifLogViewer},
        panel::status::Status,
        popup::Popups,
    },
    window::{settings::WindowSettings, settings_editor::WindowSettingsEditor},
};

#[enum_dispatch(DockWindows)]
pub trait DockWindow: Copy {
    fn title(self) -> String;
    fn ui(self, params: &mut PanelParams, ui: &mut egui::Ui);
    fn allowed_in_windows(self) -> bool {
        true
    }
    fn closeable(self) -> bool {
        true
    }
}

#[enum_dispatch]
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "ty")]
pub enum DockWindows {
    Tilemap,
    ComponentEditor,
    ProjectEditor,
    WindowSettingsEditor,
    TileSettingsEditor,
    KeymapSettingsEditor,
    MiscSettingsEditor,
    NotifLogViewer,
    ComponentList,
    HistoryViewer,
}

#[derive(Clone, Resource)]
pub struct DockLayout(pub DockState<DockWindows>);

impl Default for DockLayout {
    fn default() -> Self {
        let mut state = DockState::new(vec![Tilemap.into()]);
        let tree = state.main_surface_mut();
        let [_, _] = tree.split_left(NodeIndex::root(), 0.2, vec![ComponentEditor.into()]);
        let [_, _] = tree.split_right(
            NodeIndex::root(),
            0.8,
            vec![
                ProjectEditor.into(),
                ComponentList.into(),
                HistoryViewer.into(),
            ],
        );
        Self(state)
    }
}
impl DockLayout {
    pub fn load() -> Self {
        if !data_path("dock_layout.msgpack").exists() {
            let s = Self::default();
            let _ = s.save();
            return s;
        }
        match load_msgpack(&data_path("dock_layout.msgpack"), Some("dock layout")) {
            Ok(str) => {
                info!("Found dock layout file");
                Self(str)
            }
            Err(e) => {
                info!("Couldn't open or parse dock layout file: {e:?}");

                Self::default()
            }
        }
    }
    pub fn save(&self) -> eyre::Result<()> {
        save_msgpack(
            &self.0,
            &data_path("dock_layout.msgpack"),
            Some("dock layout"),
        )
    }
}

impl DockLayout {
    fn ui(&mut self, params: &mut PanelParams, ctx: &egui::Context) {
        DockArea::new(&mut self.0)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, params);
    }
}

#[derive(SystemParam)]
#[non_exhaustive]
pub struct PanelParams<'w, 's> {
    pub queries: ParamSet<
        'w,
        's,
        (
            Query<'w, 's, (Entity, &'static mut PlaComponent), With<SelectedComponent>>,
            Query<'w, 's, (Entity, &'static PlaComponent)>,
        ),
    >,
    pub camera: Query<'w, 's, &'static mut Transform, With<Camera>>,
    pub commands: Commands<'w, 's>,
    pub skin: Res<'w, Skin>,
    pub editor_state: Res<'w, State<EditorState>>,
    pub window_settings: ResMut<'w, WindowSettings>,
    pub tile_settings: ResMut<'w, TileSettings>,
    pub keymap_settings: ResMut<'w, KeymapSettings>,
    pub misc_settings: ResMut<'w, MiscSettings>,
    pub status: ResMut<'w, Status>,
    pub popups: ResMut<'w, Popups>,
    pub namespaces: ResMut<'w, Namespaces>,
    pub new_namespace: Local<'s, String>,
    pub history: ResMut<'w, History>,
    pub mouse_pos_world: Res<'w, MousePosWorld>,
    pub pending_tiles: Res<'w, PendingTiles>,
    pub zoom: Res<'w, Zoom>,
}

impl egui_dock::TabViewer for PanelParams<'_, '_> {
    type Tab = DockWindows;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(self, ui);
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        tab.closeable()
    }

    fn tab_style_override(&self, tab: &Self::Tab, global_style: &TabStyle) -> Option<TabStyle> {
        matches!(tab, DockWindows::Tilemap(_)).then(|| TabStyle {
            tab_body: TabBodyStyle {
                inner_margin: egui::Margin::ZERO,
                ..global_style.tab_body
            },
            ..global_style.to_owned()
        })
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        tab.allowed_in_windows()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, DockWindows::Tilemap(_))
    }
}

pub fn open_dock_window<W: DockWindow + Into<DockWindows>>(state: &mut DockLayout, window: W) {
    let a = state
        .0
        .iter_all_tabs()
        .find(|(_, a)| a.title() == window.title());
    if let Some((a, _)) = a {
        info!("Focusing on {}", window.title());
        let a = a.to_owned();
        state.0.set_focused_node_and_surface(a);
    } else {
        info!("Creating new window {}", window.title());
        state.0.add_window(vec![window.into()]);
    }
}

pub fn panel_sy(
    mut state: ResMut<DockLayout>,
    mut ctx: EguiContexts,
    mut params: PanelParams,
    mut tick: Local<u8>,
) {
    let Ok(ctx) = ctx.ctx_mut() else {
        return;
    };
    state.ui(&mut params, ctx);
    *tick = tick.overflowing_add(1).0;
    if (*tick).is_multiple_of(64) {
        let _ = state.save();
    }
}

#[derive(Clone, Copy, Event)]
pub struct ResetPanelDockStateEv;

pub fn on_reset_panel(_trigger: Trigger<ResetPanelDockStateEv>, mut state: ResMut<DockLayout>) {
    NOTIF_LOG.push("Layout reset", ToastLevel::Success);
    *state = DockLayout::default();
    let _ = state.save();
}
