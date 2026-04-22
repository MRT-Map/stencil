use std::sync::LazyLock;

use egui_layout_job_macro::{layout_job, text_format};
use itertools::Itertools;
use tracing::{debug, info};

use crate::{
    App,
    mode::EditorMode,
    project::{history::Events, pla3::PlaComponent},
};

static EM: LazyLock<egui::TextFormat> = LazyLock::new(|| text_format!(yellow));
static AC: LazyLock<egui::TextFormat> =
    LazyLock::new(|| text_format!(white, bg_black, expand_bg[2.0]));
static CD: LazyLock<egui::TextFormat> = LazyLock::new(|| text_format!(mono));
macro_rules! ac {
    (l-click) => {
        ("L-click", 1.0, AC.clone())
    };
    (m-click) => {
        ("M-click", 1.0, AC.clone())
    };
    (r-click) => {
        ("R-click", 1.0, AC.clone())
    };
    (l-click2) => {
        ("Dbl-L-click", 1.0, AC.clone())
    };
    (m-click2) => {
        ("Dbl-M-click", 1.0, AC.clone())
    };
    (r-click2) => {
        ("Dbl-R-click", 1.0, AC.clone())
    };
    (shift) => {
        ("Shift", 1.0, AC.clone())
    };
    (alt) => {
        ("Alt", 1.0, AC.clone())
    };
    (cmd) => {
        (
            if cfg!(target_os = "macos") {
                "Cmd"
            } else {
                "Ctrl"
            },
            1.0,
            AC.clone(),
        )
    };
    ($action:expr) => {
        (
            &shortcut_settings.format_action($action, ctx),
            1.0,
            AC.clone(),
        )
    };
}
macro_rules! cm {
    ($components:expr) => {{
        const COMPONENT_THRESHOLD: usize = 5;
        let components = $components;
        if components.len() > COMPONENT_THRESHOLD {
            (&format!("{} components", components.len()), 1.0, EM.clone())
        } else {
            (
                &components.iter().map(ToString::to_string).join(" "),
                1.0,
                CD.clone(),
            )
        }
    }};
}

impl App {
    pub fn status_init(&mut self) {
        if self.ui.status.is_empty() {
            self.status_default();
        }
    }

    pub fn status_on_copy(&mut self) {
        if self.ui.map.clipboard.is_empty() {
            info!("Nothing to copy");
            self.ui.status = layout_job!("Nothing to copy");
        } else {
            let components = &self.ui.map.clipboard;
            info!(ids=?components
                .iter()
                .map(|a| &a.full_id)
                .collect::<Vec<_>>(), "Copied components");
            self.ui.status = layout_job!("Copied " #cm!(components));
        }
    }
    pub fn status_on_cut(&mut self) {
        if self.ui.map.clipboard.is_empty() {
            info!("Nothing to cut");
            self.ui.status = layout_job!("Nothing to cut");
        } else {
            let components = &self.ui.map.clipboard;
            info!(ids=?components
                .iter()
                .map(|a| &a.full_id)
                .collect::<Vec<_>>(), "Cut components");
            self.ui.status = layout_job!("Cut " #cm!(components));
        }
    }
    pub fn status_on_paste(&mut self, components: &[PlaComponent]) {
        if components.is_empty() {
            info!("Nothing to paste");
            self.ui.status = layout_job!("Nothing to paste");
        } else {
            info!(ids=?components.iter()
                .map(|a| &a.full_id)
                .collect::<Vec<_>>(), "Pasted and selected components");
            self.ui.status = layout_job!("Pasted " #cm!(components));
        }
    }
    pub fn status_on_clear_clipboard(&mut self) {
        info!("Cleared clipboard");
        self.ui.status = layout_job!("Cleared clipboard");
    }
    pub fn status_on_delete(&mut self, components: &[PlaComponent]) {
        if components.is_empty() {
            info!("Nothing to delete");
            self.ui.status = layout_job!("Nothing to delete");
        } else {
            info!(ids=?components
                .iter()
                .map(|a| &a.full_id)
                .collect::<Vec<_>>(), "Deleted components");
            self.ui.status = layout_job!("Deleted " #cm!(components));
        }
    }

    pub fn status_on_create(&mut self, ty: &str, component: &PlaComponent) {
        info!(%component, "Created new {ty}");
        debug!(?component);
        self.ui.status = layout_job!(format!("Created new {ty} ") @[CD](component.full_id));
    }

    pub fn status_on_move(&mut self, delta: geo::Coord<i32>) {
        self.ui.status = layout_job!("Move selected components by " @[CD](delta.x ", " delta.y));
    }
    pub fn status_on_move_finish(&mut self, delta: geo::Coord<i32>) {
        self.ui.status =
            layout_job!("Finished moving selected components by " @[CD](delta.x ", " delta.y));
    }

    pub fn status_undo(&mut self, event: &Events) {
        self.ui.status = layout_job!("Undid " @[EM](event));
    }

    pub fn status_redo(&mut self, event: &Events) {
        self.ui.status = layout_job!("Redid " @[EM](event));
    }

    pub fn status_select(&mut self) {
        if self.ui.map.selected.is_empty() {
            self.status_default();
            return;
        }
        self.ui.status = layout_job!("Selecting " #cm!(&self.map_selected_components()));
    }

    pub fn status_default(&mut self) {
        self.ui.status = match self.mode {
            EditorMode::Select => {
                layout_job!(@[EM]("Select: ") #ac!(l-click) " to select component (" #ac!(shift) " to select multiple). " #ac!(m-click) " and drag, or (" #ac!(shift) " and) scroll to pan. " #ac!(cmd) " and scroll to zoom.")
            }
            EditorMode::Nodes => {
                layout_job!(@[EM]("Editing nodes: ") #ac!(r-click) " and drag circle to create/move node. " #ac!(r-click) " large circle without dragging to delete node." #ac!(r-click) " anywhere else on a selected component to move it.")
            }
            EditorMode::CreatePoint => {
                layout_job!(@[EM]("Creating points: ") #ac!(l-click) " to create point.")
            }
            EditorMode::CreateLine => {
                layout_job!(@[EM]("Creating lines: ") #ac!(l-click) " to start and continue line " #ac!(r-click) " to undo. " #ac!(l-click2) " to end at pointer, " #ac!(m-click2) " to end at last node." #ac!(shift) " to create bézier curves. " #ac!(cmd) " to snap to angle.")
            }
            EditorMode::CreateArea => {
                layout_job!(@[EM]("Creating areas: ") #ac!(l-click) " to start and continue line " #ac!(r-click) " to undo. " #ac!(l-click2) " to end at pointer, " #ac!(m-click2) " to end at last node." #ac!(shift) " to create bézier curves. " #ac!(cmd) " to snap to angle.")
            }
        };
    }
}
