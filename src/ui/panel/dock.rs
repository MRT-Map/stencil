use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{egui, EguiContexts};
use egui_dock::{DockArea, DockState, NodeIndex, Style};

use crate::{
    misc::Action,
    pla2::{
        bundle::SelectedComponent,
        component::{EditorCoords, PlaComponent},
        skin::Skin,
    },
    ui::panel::component_panel::{component_ui, PrevNamespaceUsed},
};

#[derive(Debug)]
#[non_exhaustive]
enum DockWindow {
    Tilemap,
    ComponentEditor,
}

#[derive(Resource)]
pub struct PanelDockState {
    state: DockState<DockWindow>,
    pub viewport_rect: egui::Rect,
    pub layer_id: egui::LayerId,
}

impl Default for PanelDockState {
    fn default() -> Self {
        let mut state = DockState::new(vec![DockWindow::Tilemap]);
        let tree = state.main_surface_mut();
        let [_, _inspector] =
            tree.split_left(NodeIndex::root(), 0.15, vec![DockWindow::ComponentEditor]);

        Self {
            state,
            layer_id: egui::LayerId::background(),
            viewport_rect: egui::Rect::NOTHING,
        }
    }
}

impl PanelDockState {
    fn ui(&mut self, params: PanelParams, ctx: &mut egui::Context) {
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

#[derive(Resource)]
pub struct TempUi<'a>(pub &'a mut egui::Ui);

struct TabViewer<'a, 'w, 's> {
    params: PanelParams<'w, 's>,
    viewport_rect: &'a mut egui::Rect,
    layer_id: &'a mut egui::LayerId,
}

impl egui_dock::TabViewer for TabViewer<'_, '_, '_> {
    type Tab = DockWindow;

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        match window {
            DockWindow::Tilemap => "Map",
            DockWindow::ComponentEditor => "Component",
        }
        .into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        match window {
            DockWindow::Tilemap => {
                *self.layer_id = ui.layer_id();
                *self.viewport_rect = ui.clip_rect();
            }
            DockWindow::ComponentEditor => {
                component_ui(ui, &mut self.params);
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, DockWindow::Tilemap)
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, DockWindow::Tilemap)
    }
}

#[derive(SystemParam)]
pub struct PanelParams<'w, 's> {
    pub selected:
        Query<'w, 's, (Entity, &'static mut PlaComponent<EditorCoords>), With<SelectedComponent>>,
    pub commands: Commands<'w, 's>,
    pub skin: Res<'w, Skin>,
    pub prev_namespace_used: ResMut<'w, PrevNamespaceUsed>,
    pub actions: EventWriter<'w, Action>,
}

pub fn panel_sy(mut state: ResMut<PanelDockState>, mut ctx: EguiContexts, params: PanelParams) {
    state.ui(params, ctx.ctx_mut());
}

#[must_use]
pub fn within_tilemap(ctx: &mut EguiContexts, panel: &Res<PanelDockState>) -> bool {
    ctx.ctx_mut()
        .rect_contains_pointer(panel.layer_id, panel.viewport_rect)
}
