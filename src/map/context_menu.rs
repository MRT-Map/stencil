use crate::{App, map::MapWindow, shortcut::ShortcutAction};

impl MapWindow {
    pub const HOVERED_OVER_CTX_MENU: &str = "hovered over context menu";
    pub fn component_context_menu(app: &mut App, response: &egui::Response) {
        if app.mode.is_editing() {
            return;
        }

        let Some(ctx_menu_response) = response
            .context_menu(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                macro_rules! button {
                    ($ui:ident, $label:literal, $action:expr, $f:block) => {
                        if app.menu_button_fn("context menu", $ui, $label, $action) {
                            $f
                        }
                    };
                    ($ui:ident, $label:literal, $action:expr, window $w:expr) => {
                        app.menu_button_window("context menu", $ui, $label, $action, $w);
                    };
                }
                if !app.ui.map.selected_components.is_empty() {
                    button!(ui, "Copy", Some(ShortcutAction::Copy), {
                        app.copy_selected_components(&response.ctx);
                    });
                    button!(ui, "Cut", Some(ShortcutAction::Cut), {
                        app.cut_selected_components(&response.ctx);
                    });
                    button!(ui, "Delete", Some(ShortcutAction::Delete), {
                        app.delete_selected_components(&response.ctx);
                    });
                    ui.separator();
                }
                button!(ui, "Paste", Some(ShortcutAction::Paste), {
                    app.paste_clipboard_components(&response.ctx);
                });
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
            )
        });
    }
}
