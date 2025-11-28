use tracing::info;

use crate::{App, map::MapWindow, shortcut::ShortcutAction};

impl MapWindow {
    pub fn component_context_menu(&mut self, app: &mut App, response: &egui::Response) {
        if app.mode.is_editing() {
            return;
        }

        response.context_menu(|ui| {
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
            if !self.selected_components.is_empty() {
                button!(ui, "Copy", Some(ShortcutAction::Copy), {
                    self.copy_selected_components(app);
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
        });
    }
}
