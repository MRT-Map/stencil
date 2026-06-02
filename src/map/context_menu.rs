use crate::{App, map::MapWindow, shortcut::ShortcutAction};

impl MapWindow {
    pub const HOVERED_OVER_CTX_MENU: &str = "hovered over context menu";
    #[tracing::instrument(skip_all)]
    pub fn component_context_menu(app: &mut App, response: &egui::Response) {
        if app.mode.is_editing() {
            return;
        }

        let Some(ctx_menu_response) = response
            .context_menu(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                macro_rules! button {
                    ($ui:ident, $label:literal, $action:expr) => {
                        if app.menu_button_fn("context menu", $ui, $label, Some($action)) {
                            app.run_action($ui, $action, None);
                        }
                    };
                }
                if !app.ui.map.selected.is_empty() {
                    button!(ui, "Copy", ShortcutAction::Copy);
                    button!(ui, "Cut", ShortcutAction::Cut);
                    button!(ui, "Delete", ShortcutAction::Delete);
                    ui.separator();
                }
                button!(ui, "Paste", ShortcutAction::Paste);
            })
            .map(|a| a.response)
        else {
            response
                .ctx
                .data_mut(|a| a.insert_temp(Self::HOVERED_OVER_CTX_MENU.into(), false));
            return;
        };

        response.ctx.data_mut(|a| {
            a.insert_temp(
                Self::HOVERED_OVER_CTX_MENU.into(),
                ctx_menu_response.contains_pointer(),
            );
        });
    }
}
