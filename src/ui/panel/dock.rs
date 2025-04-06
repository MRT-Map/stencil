use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{egui, EguiContexts};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabBodyStyle, TabStyle};
use egui_file_dialog::{FileDialog, FileDialogStorage};
use egui_notify::ToastLevel;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use crate::{
    component::{
        panels::{component_editor::ComponentEditor, component_list::ComponentList},
        pla2::{EditorCoords, PlaComponent},
        skin::Skin,
    },
    history::{history_viewer::HistoryViewer, History},
    keymaps::{settings::KeymapSettings, settings_editor::KeymapSettingsEditor},
    misc_config::{settings::MiscSettings, settings_editor::MiscSettingsEditor},
    project::{project_editor::ProjectEditor, Namespaces},
    state::EditorState,
    tile::zoom::Zoom,
    ui::{
        cursor::mouse_pos::MousePosWorld,
        notif::{viewer::NotifLogViewer, NotifLogRwLockExt, NOTIF_LOG},
        panel::status::Status,
        popup::Popup,
        tilemap::{
            settings::{Basemap, TileSettings},
            settings_editor::TileSettingsEditor,
            tile::PendingTiles,
            window::Tilemap,
        },
    },
    window::{settings::WindowSettings, settings_editor::WindowSettingsEditor},
};
use crate::component::actions::selecting::SelectedComponent;
use crate::dirs_paths::{cache_path, data_path};
use crate::file::{load_toml, save_toml, save_toml_with_header};
use crate::ui::tilemap::window::PointerWithinTilemap;

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
        if !data_path("dock_layout.toml").exists() {
            let s = Self::default();
            let _ = s.save();
            return s;
        }
        match load_toml(&data_path("dock_layout.toml"), Some("dock layout")) {
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
        save_toml(&self.0, &data_path("dock_layout.toml"), Some("dock layout"))
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
            Query<
                'w,
                's,
                (Entity, &'static mut PlaComponent<EditorCoords>),
                With<SelectedComponent>,
            >,
            Query<'w, 's, &'static PlaComponent<EditorCoords>>,
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
    pub popup: EventWriter<'w, Popup>,
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


pub fn open_dock_window<W: DockWindow + Into<DockWindows>>(
    state: &mut DockLayout,
    window: W,
) {
    let a = state.0
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

pub fn panel_sy(mut state: ResMut<DockLayout>, mut ctx: EguiContexts, mut params: PanelParams) {
    let Some(ctx) = ctx.try_ctx_mut() else {
        return;
    };
    state.ui(&mut params, ctx);
    let _ = state.save();
}

#[derive(Clone, Copy, Event)]
pub struct ResetPanelDockStateEv;

pub fn on_reset_panel(_trigger: Trigger<ResetPanelDockStateEv>, mut state: ResMut<DockLayout>) {
    NOTIF_LOG.push(&"Layout reset", ToastLevel::Success);
    *state = DockLayout::default();
}