pub trait ResponsePointerExt {
    fn dragged_by2(&self, pointer: egui::PointerButton) -> bool;
    fn drag_started_by2(&self, pointer: egui::PointerButton) -> bool;
    fn drag_stopped_by2(&self, pointer: egui::PointerButton) -> bool;
    fn clicked_by2(&self, pointer: egui::PointerButton) -> bool;
    fn double_clicked_by2(&self, pointer: egui::PointerButton) -> bool;
}

impl ResponsePointerExt for egui::Response {
    fn dragged_by2(&self, pointer: egui::PointerButton) -> bool {
        match pointer {
            egui::PointerButton::Primary => {
                self.dragged_by(egui::PointerButton::Primary)
                    && !self.ctx.input(|i| i.modifiers.alt)
            }
            egui::PointerButton::Middle => {
                self.dragged_by(egui::PointerButton::Middle)
                    || (self.dragged_by(egui::PointerButton::Primary)
                        && self.ctx.input(|i| i.modifiers.alt))
            }
            pointer => self.dragged_by(pointer),
        }
    }
    fn drag_started_by2(&self, pointer: egui::PointerButton) -> bool {
        match pointer {
            egui::PointerButton::Primary => {
                self.drag_started_by(egui::PointerButton::Primary)
                    && !self.ctx.input(|i| i.modifiers.alt)
            }
            egui::PointerButton::Middle => {
                self.drag_started_by(egui::PointerButton::Middle)
                    || (self.drag_started_by(egui::PointerButton::Primary)
                        && self.ctx.input(|i| i.modifiers.alt))
            }
            pointer => self.drag_started_by(pointer),
        }
    }
    fn drag_stopped_by2(&self, pointer: egui::PointerButton) -> bool {
        match pointer {
            egui::PointerButton::Primary => {
                self.drag_stopped_by(egui::PointerButton::Primary)
                    && !self.ctx.input(|i| i.modifiers.alt)
            }
            egui::PointerButton::Middle => {
                self.drag_stopped_by(egui::PointerButton::Middle)
                    || (self.drag_stopped_by(egui::PointerButton::Primary)
                        && self.ctx.input(|i| i.modifiers.alt))
            }
            pointer => self.drag_stopped_by(pointer),
        }
    }
    fn clicked_by2(&self, pointer: egui::PointerButton) -> bool {
        match pointer {
            egui::PointerButton::Primary => {
                self.clicked_by(egui::PointerButton::Primary)
                    && !self.ctx.input(|i| i.modifiers.alt)
            }
            egui::PointerButton::Middle => {
                self.clicked_by(egui::PointerButton::Middle)
                    || (self.clicked_by(egui::PointerButton::Primary)
                        && self.ctx.input(|i| i.modifiers.alt))
            }
            pointer => self.clicked_by(pointer),
        }
    }
    fn double_clicked_by2(&self, pointer: egui::PointerButton) -> bool {
        match pointer {
            egui::PointerButton::Primary => {
                self.double_clicked_by(egui::PointerButton::Primary)
                    && !self.ctx.input(|i| i.modifiers.alt)
            }
            egui::PointerButton::Middle => {
                self.double_clicked_by(egui::PointerButton::Middle)
                    || (self.double_clicked_by(egui::PointerButton::Primary)
                        && self.ctx.input(|i| i.modifiers.alt))
            }
            pointer => self.double_clicked_by(pointer),
        }
    }
}
