use crate::{TitoApp, emulator::CtrlMSG};

impl TitoApp {
    pub fn editor_toolbar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.button("Compile").clicked() {
            self.current_prog = self.editor.compile();
            self.emu_tx.send(CtrlMSG::LoadProg(self.current_prog.clone()));
        }
    }

    pub fn editor_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let rowheight = 14;
        let rowcount = ui.available_height() as i32 / rowheight - 1;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.add_enabled_ui(false, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.editor.line_no)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(rowcount as usize)
                            .lock_focus(true)
                            .desired_width(22.),
                    ); //.layouter(&mut layouter),
                });
                if ui
                    .add(
                        egui::TextEdit::multiline(&mut self.editor.source_code)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(rowcount as usize)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY), //.layouter(&mut layouter),
                    )
                    .changed()
                {
                    self.editor.update_linecount()
                }
            });
        });
    }
}
