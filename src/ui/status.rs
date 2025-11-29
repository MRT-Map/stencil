use std::collections::HashMap;

use indexmap::IndexMap;
use itertools::Itertools;
use tracing::{debug, info};

use crate::{
    App,
    mode::EditorMode,
    project::{
        history::Events,
        pla3::{FullId, PlaComponent},
    },
    shortcut::{ShortcutAction, settings::ShortcutSettings},
};

enum Section<'a> {
    Text(&'a str),
    Emphasis(&'a str),
    Mouse(&'a str),
    Modifier(&'a str),
    ShortcutAction(ShortcutAction),
    Code(&'a str),
}
impl Section<'_> {
    const LEADING_SPACE: f32 = 1.0;
    fn format(
        sections: impl IntoIterator<Item = Self>,
        shortcut_settings: &mut ShortcutSettings,
        ctx: &egui::Context,
    ) -> egui::text::LayoutJob {
        let action_default_format = egui::TextFormat {
            background: egui::Color32::BLACK,
            expand_bg: 2.0,
            color: egui::Color32::WHITE,
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        for section in sections {
            match section {
                Self::Text(s) => job.append(s, Self::LEADING_SPACE, egui::TextFormat::default()),
                Self::Emphasis(s) => job.append(
                    s,
                    Self::LEADING_SPACE,
                    egui::TextFormat {
                        color: egui::Color32::YELLOW,
                        ..egui::TextFormat::default()
                    },
                ),
                Self::Mouse(c) => job.append(
                    &format!("{c}-click"),
                    Self::LEADING_SPACE,
                    action_default_format.clone(),
                ),
                Self::Modifier(modifier) => {
                    job.append(modifier, Self::LEADING_SPACE, action_default_format.clone());
                }
                Self::ShortcutAction(action) => job.append(
                    &shortcut_settings.format_action(action, ctx),
                    Self::LEADING_SPACE,
                    action_default_format.clone(),
                ),
                Self::Code(code) => job.append(
                    code,
                    Self::LEADING_SPACE,
                    egui::TextFormat {
                        font_id: egui::FontId {
                            family: egui::FontFamily::Monospace,
                            ..egui::FontId::default()
                        },
                        ..egui::TextFormat::default()
                    },
                ),
            }
        }
        job
    }
}
macro_rules! s {
    (em $text:expr) => {
        Section::Emphasis($text)
    };
    (l-click) => {
        Section::Mouse("L")
    };
    (m-click) => {
        Section::Mouse("M")
    };
    (r-click) => {
        Section::Mouse("R")
    };
    (l-click2) => {
        Section::Mouse("Dbl-L")
    };
    (m-click2) => {
        Section::Mouse("Dbl-M")
    };
    (r-click2) => {
        Section::Mouse("Dbl-R")
    };
    (shift) => {
        Section::Modifier("Shift")
    };
    (cmd) => {
        Section::Modifier(if cfg!(target_os = "macos") {"Cmd"} else {"Ctrl"})
    };
    (alt) => {
        Section::Modifier("Alt")
    };
    (ac $action:expr) => {
        Section::ShortcutAction($action)
    };
    (cd $code:expr) => {
        Section::Code($code)
    };
    (tx $text:expr) => {
        Section::Text($text)
    };
    (cm $components:expr) => {{
        const COMPONENT_THRESHOLD: usize = 5;
        if $components.len() > COMPONENT_THRESHOLD {
            s!(em &format!("{} components", $components.len()))
        } else {
            s!(cd &$components.iter().map(ToString::to_string).join(" "))
        }
    }};
    ($app:ident, $ctx:ident, $($section:expr),+) => {
        Section::format([$($section),+], &mut $app.shortcut_settings, $ctx).into()
    };
}

impl App {
    pub fn status_init(&mut self, ctx: &egui::Context) {
        if self.ui.status.is_empty() {
            self.status_default(ctx);
        }
    }

    pub fn status_on_copy(&mut self, ctx: &egui::Context) {
        if self.ui.map.clipboard.is_empty() {
            info!("Nothing to copy");
            self.ui.status = s!(self, ctx, s!(tx "Nothing to copy"));
        } else {
            let components = &self.ui.map.clipboard;
            info!(ids=?components
                .iter()
                .map(|a| &a.full_id)
                .collect::<Vec<_>>(), "Copied components");
            let c = s!(cm components);
            self.ui.status = s!(self, ctx, s!(tx "Copied "), c);
        }
    }
    pub fn status_on_cut(&mut self, ctx: &egui::Context) {
        if self.ui.map.clipboard.is_empty() {
            info!("Nothing to cut");
            self.ui.status = s!(self, ctx, s!(tx "Nothing to cut"));
        } else {
            let components = &self.ui.map.clipboard;
            info!(ids=?components
                .iter()
                .map(|a| &a.full_id)
                .collect::<Vec<_>>(), "Cut components");
            let c = s!(cm components);
            self.ui.status = s!(self, ctx, s!(tx "Cut "), c);
        }
    }
    pub fn status_on_paste(&mut self, components: &[PlaComponent], ctx: &egui::Context) {
        if components.is_empty() {
            info!("Nothing to paste");
            self.ui.status = s!(self, ctx, s!(tx "Nothing to paste"));
        } else {
            info!(ids=?components.iter()
                .map(|a| &a.full_id)
                .collect::<Vec<_>>(), "Pasted and selected components");
            let c = s!(cm components);
            self.ui.status = s!(self, ctx, s!(tx "Pasted "), c);
        }
    }
    pub fn status_on_delete<'a>(&mut self, components: &[PlaComponent], ctx: &egui::Context) {
        if components.is_empty() {
            info!("Nothing to delete");
            self.ui.status = s!(self, ctx, s!(tx "Nothing to delete"));
        } else {
            info!(ids=?components
                .iter()
                .map(|a| &a.full_id)
                .collect::<Vec<_>>(), "Deleted components");
            let c = s!(cm components);
            self.ui.status = s!(self, ctx, s!(tx "Deleted "), c);
        }
    }

    pub fn status_on_create(&mut self, ty: &str, component: &PlaComponent, ctx: &egui::Context) {
        info!(%component, "Created new {ty}");
        debug!(?component);
        self.ui.status = s!(
            self,
            ctx,
            s!(tx & format!("Created new {ty}")),
            s!(cd & component.full_id.to_string())
        );
    }

    pub fn status_on_move(&mut self, delta: geo::Coord<i32>, ctx: &egui::Context) {
        self.ui.status = s!(
            self,
            ctx,
            s!(tx "Moving selected components by "),
            s!(cd & delta.x.to_string()),
            s!(tx ", "),
            s!(cd & delta.y.to_string())
        );
    }
    pub fn status_on_move_finish(&mut self, delta: geo::Coord<i32>, ctx: &egui::Context) {
        self.ui.status = s!(
            self,
            ctx,
            s!(tx "Finished moving selected components by "),
            s!(cd & delta.x.to_string()),
            s!(tx ", "),
            s!(cd & delta.y.to_string())
        );
    }

    pub fn status_undo(&mut self, event: &Events, ctx: &egui::Context) {
        self.ui.status = s!(self, ctx, s!(tx "Undid "), s!(em & event.to_string()));
    }

    pub fn status_redo(&mut self, event: &Events, ctx: &egui::Context) {
        self.ui.status = s!(self, ctx, s!(tx "Redid "), s!(em & event.to_string()));
    }

    pub fn status_select(&mut self, ctx: &egui::Context) {
        if self.ui.map.selected_components.is_empty() {
            self.status_default(ctx);
            return;
        }
        let c = s!(cm & self.map_selected_components());
        self.ui.status = s!(self, ctx, s!(tx "Selecting "), c);
    }

    pub fn status_default(&mut self, ctx: &egui::Context) {
        self.ui.status = match self.mode {
            EditorMode::Select => s!(
                self,
                ctx,
                s!(em "Select: "),
                s!(l - click),
                s!(tx " to select component ("),
                s!(shift),
                s!(tx " to select multiple). "),
                s!(m - click),
                s!(tx " and drag, or ("),
                s!(shift),
                s!(tx " and) scroll to pan. "),
                s!(cmd),
                s!(tx " and scroll to zoom.")
            ),
            EditorMode::Nodes => s!(
                self,
                ctx,
                s!(em "Editing nodes: "),
                s!(r - click),
                s!(tx " and drag circle to create/move node. "),
                s!(r - click),
                s!(tx " large circle without dragging to delete node."),
                s!(r - click),
                s!(tx " anywhere else on a selected component to move it.")
            ),
            EditorMode::CreatePoint => s!(
                self,
                ctx,
                s!(em "Creating points: "),
                s!(l - click),
                s!(tx " to create point.")
            ),
            EditorMode::CreateLine => s!(
                self,
                ctx,
                s!(em "Creating lines: "),
                s!(l - click),
                s!(tx " to start and continue line "),
                s!(r - click),
                s!(tx " to undo. "),
                s!(l - click2),
                s!(tx " to end at pointer, "),
                s!(m - click2),
                s!(tx " to end at last node."),
                s!(shift),
                s!(tx " to create bézier curves. "),
                s!(alt),
                s!(tx " to snap to angle.")
            ),
            EditorMode::CreateArea => s!(
                self,
                ctx,
                s!(em "Creating areas: "),
                s!(l - click),
                s!(tx " to start and continue line "),
                s!(r - click),
                s!(tx " to undo. "),
                s!(l - click2),
                s!(tx " to end at pointer, "),
                s!(m - click2),
                s!(tx " to end at last node."),
                s!(shift),
                s!(tx " to create bézier curves. "),
                s!(alt),
                s!(tx " to snap to angle.")
            ),
        };
    }
}
