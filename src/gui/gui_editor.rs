use crate::TitoApp;
use egui::{Color32, FontId, RichText};

const FONT_COMPILER: FontId = FontId::monospace(12.0);
const COL_TEXT: Color32 = Color32::DARK_GRAY;
const COL_TEXT_HI: Color32 = Color32::WHITE;

impl TitoApp {
    pub fn editor_toolbar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.button("Compile").clicked() {
            self.file_compile();
        }
    }

    pub fn editor_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::TopBottomPanel::bottom("compiler_output")
            .default_height(0.)
            .resizable(true)
            .min_height(16.)
            .max_height(ui.available_height() - 64.) // For some reason the top slice becomes invisible.
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Compiler output:")
                        .font(FONT_COMPILER)
                        .color(COL_TEXT),
                );
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([true, false])
                    .min_scrolled_height(0.)
                    .show(ui, |ui| {
                        ui.allocate_space(egui::Vec2 {
                            x: ui.available_width(),
                            y: 0.,
                        });
                        let rowheight = 14;
                        let rowcount = (ui.available_height() as i32) / rowheight + 1;
                        ui.add_enabled(
                            false,
                            egui::TextEdit::multiline(&mut self.editor.compiler.output)
                                .font(FONT_COMPILER) // for cursor height
                                //.code_editor()
                                .desired_rows(rowcount as usize)
                                //.lock_focus(true)
                                .interactive(false)
                                .desired_width(f32::INFINITY),
                        );
                        //ui.label(RichText::new(&self.editor.compiler.output).font(FONT_COMPILER));
                    });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            let rowheight = 14;
            let rowcount = (ui.available_height() as i32 + self.editor.linecnt) / rowheight + 2;
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
        });
    }
}
