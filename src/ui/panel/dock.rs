use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{egui, EguiContexts};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabBodyStyle, TabStyle};
use egui_file_dialog::FileDialog;
use egui_notify::ToastLevel;
use enum_dispatch::enum_dispatch;

use crate::{
    component::{
        bundle::SelectedComponent,
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

#[enum_dispatch(DockWindows)]
pub trait DockWindow: Copy {
    fn title(self) -> String;
    fn ui(self, tab_viewer: &mut TabViewer, ui: &mut egui::Ui);
    fn allowed_in_windows(self) -> bool {
        true
    }
    fn closeable(self) -> bool {
        true
    }
}

#[enum_dispatch]
#[derive(Clone, Copy)]
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

#[derive(Resource)]
pub struct PanelDockState {
    pub state: DockState<DockWindows>,
    pub viewport_rect: egui::Rect,
    pub layer_id: egui::LayerId,
}

impl Default for PanelDockState {
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

        Self {
            state,
            layer_id: egui::LayerId::background(),
            viewport_rect: egui::Rect::NOTHING,
        }
    }
}

impl PanelDockState {
    fn ui(&mut self, params: &mut PanelParams, ctx: &egui::Context) {
        let mut tab_viewer = TabViewer {
            params,
            viewport_rect: &mut self.viewport_rect,
            layer_id: &mut self.layer_id,
        };

        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

pub struct TabViewer<'a, 'w, 's> {
    pub params: &'a mut PanelParams<'w, 's>,
    pub viewport_rect: &'a mut egui::Rect,
    pub layer_id: &'a mut egui::LayerId,
}

pub struct FileDialogs {
    pub tile_settings_import: FileDialog,
    pub tile_settings_export: Option<(Basemap, FileDialog)>,
    pub project_select: FileDialog,
}

impl Default for FileDialogs {
    fn default() -> Self {
        Self {
            tile_settings_import: TileSettingsEditor::import_dialog(),
            tile_settings_export: None,
            project_select: ProjectEditor::select_dialog(),
        }
    }
}

impl egui_dock::TabViewer for TabViewer<'_, '_, '_> {
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

pub fn window_action_handler<W: DockWindow + Into<DockWindows>>(
    state: &mut PanelDockState,
    window: W,
) {
    let a = state
        .state
        .iter_all_tabs()
        .find(|(_, a)| a.title() == window.title());
    if let Some((a, _)) = a {
        info!("Focusing on {}", window.title());
        let a = a.to_owned();
        state.state.set_focused_node_and_surface(a);
    } else {
        info!("Creating new window {}", window.title());
        state.state.add_window(vec![window.into()]);
    }
}

pub fn panel_sy(mut state: ResMut<PanelDockState>, mut ctx: EguiContexts, mut params: PanelParams) {
    let Some(ctx) = ctx.try_ctx_mut() else {
        return;
    };
    state.ui(&mut params, ctx);
}

#[derive(Clone, Copy, Event)]
pub struct ResetPanelDockStateEv;

#[expect(clippy::needless_pass_by_value)]
pub fn on_reset_panel(_trigger: Trigger<ResetPanelDockStateEv>, mut state: ResMut<PanelDockState>) {
    NOTIF_LOG.push(&"Layout reset", ToastLevel::Success);
    *state = PanelDockState::default();
}

#[must_use]
pub fn within_tilemap(ctx: &mut EguiContexts, panel: &Res<PanelDockState>) -> bool {
    ctx.try_ctx_mut().map_or(true, |a| {
        a.rect_contains_pointer(panel.layer_id, panel.viewport_rect)
    })
}
