use super::{super::emulator::CtrlMSG, Base};
use crate::TitoApp;
pub mod instruction_parser;
use eframe::emath::format_with_decimals_in_range;
use egui::{Button, Color32, Context, FontId, Frame, RichText, TextEdit, Ui};
use egui_extras::{Column, RetainedImage, TableBuilder};
use instruction_parser::*;
use num_traits::clamp;

const FONT_TBL: FontId = FontId::monospace(12.0);
const FONT_TBLH: FontId = FontId::proportional(12.5);
const FONT_BUT: FontId = FontId::monospace(16.0);
const COL_TEXT: Color32 = Color32::DARK_GRAY;
const COL_TEXT_HI: Color32 = Color32::WHITE;

impl TitoApp {
    pub fn emulator_toolbar(&mut self, ctx: &Context, ui: &mut Ui) {
        let text_onoff;
        match self.emu_running {
            true => text_onoff = RichText::new("⏼on/off").color(Color32::WHITE),
            false => text_onoff = RichText::new("⏼on/off"),
        }
        if ui.add(Button::new(text_onoff)).clicked() {
            self.emu_playing = false;
            self.emu_tx.send(CtrlMSG::PlayPause(false));
            if self.emu_running {
                self.emu_running = false;
                self.emu_tx.send(CtrlMSG::Stop);
            } else {
                self.emu_running = true;
                self.emu_tx.send(CtrlMSG::Start);
            }
        }

        ui.add_enabled_ui(self.emu_running, |ui| {
            let text_play;
            match self.emu_playing {
                true => text_play = RichText::new("⏸").color(Color32::WHITE),
                false => text_play = RichText::new("▶"),
            }
            if ui
                .add(Button::new(text_play).min_size(egui::vec2(24.0, 0.0)))
                .clicked()
            {
                self.emu_playing = !self.emu_playing;
                self.emu_tx.send(CtrlMSG::PlayPause(self.emu_playing));
            }
            ui.add_enabled_ui(!self.emu_playing, |ui| {
                if ui
                    .add(Button::new(RichText::new("|▶")).min_size(egui::vec2(24.0, 0.0)))
                    .clicked()
                {
                    self.emu_tx.send(CtrlMSG::Tick);
                }
            })
        });

        ui.separator();

        if ui.button("Reload").clicked() {
            self.emu_tx
                .send(CtrlMSG::LoadProg(self.current_prog.clone()));
        }

        ui.separator();
        if ui
            .selectable_label(self.emugui_display, "Show Graphics")
            .clicked()
        {
            self.emugui_display = !self.emugui_display;
        }
        ui.separator();
    }

    pub fn emulator_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        self.refresh_emu_state();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::right("register_panel")
                .resizable(false)
                .show(ctx, |ui| {
                    self.stateview(ui);
                    self.regview(ui);
                });
            // IO Panel
            egui::SidePanel::right("io_panel")
                .resizable(false)
                .max_width(128.0)
                .show(ctx, |ui| {
                    self.ioview(ctx, ui);
                    ui.separator();
                });
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.emugui_display {
                    self.display(ctx, ui);
                    self.emu_tx.send(CtrlMSG::GetDisp);
                }
                self.memview(ctx, ui);
            });
        });
    }

    fn memview(&mut self, ctx: &Context, ui: &mut Ui) {
        let width_adr: f32 = 96.0;
        let width_val: f32 = match self.mem_val_base == Base::Bin {
            true => 256.0,
            false => 96.0,
        };
        let width_ins: f32 = 192.0;

        TableBuilder::new(ui)
            .striped(true)
            .column(Column::exact(width_adr)) // Address
            .column(Column::exact(width_val)) // Value
            .column(Column::exact(width_ins)) // Instruction
            .column(Column::remainder()) // Registers PC/SP/FP
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading(RichText::new("Address").font(FONT_TBLH.clone()));
                });
                header.col(|ui| {
                    ui.heading(RichText::new("Value").font(FONT_TBLH.clone()));
                });
                header.col(|ui| {
                    ui.heading(RichText::new("Instruction").font(FONT_TBLH.clone()));
                });
                header.col(|ui| {
                    ui.heading(RichText::new("").font(FONT_TBLH.clone()));
                });
            })
            .body(|mut body| {
                let rowcount = self.emu_memory_len;
                for i in 0..rowcount {
                    let adr = self.emu_memory_off + i;
                    let val: i32 = self.emu_memory[i as usize];
                    let pc = self.emu_registers[0];
                    let ir = self.emu_registers[1];
                    let tr = self.emu_registers[2];
                    let sr = self.emu_registers[3];
                    let sp = self.emu_registers[10];
                    let fp = self.emu_registers[11];
                    // Create strings
                    let mut reg_str = String::new();
                    if pc == adr || sp == adr || fp == adr {
                        reg_str.push_str("<-- ");
                        if pc == adr {
                            reg_str.push_str("PC ")
                        }
                        if sp == adr {
                            reg_str.push_str("SP ")
                        }
                        if fp == adr {
                            reg_str.push_str("FP ")
                        }
                    }
                    let adr_str = match self.mem_adr_base {
                        Base::Bin => format!("{adr:#b}"),
                        Base::Dec => format!("{adr}"),
                        Base::Hex => format!("{adr:#x}"),
                    };
                    let val_str = match self.mem_val_base {
                        Base::Bin => format!("{val:#034b}"),
                        Base::Dec => format!("{val}"),
                        Base::Hex => format!("{val:#010x}"),
                    };
                    let ins_str = instruction_to_string(val);
                    // Decide style
                    let col = if i == pc { COL_TEXT_HI } else { COL_TEXT };
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(RichText::new(adr_str).font(FONT_TBL.clone()).color(col));
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(val_str).font(FONT_TBL.clone()).color(col));
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(ins_str).font(FONT_TBL.clone()).color(col));
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(reg_str).font(FONT_TBL.clone()).color(col));
                        });
                    });
                }
            });
    }

    fn regview(&mut self, ui: &mut Ui) {
        // CPU Registers
        ui.label("CPU Registers");
        let reg_name_width: f32 = 16.0;
        let reg_val_width: f32 = match self.regs_base == Base::Bin {
            true => 256.0,
            false => 72.0,
        };

        let pc = self.emu_registers[0];
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::initial(reg_name_width))
            .column(Column::exact(reg_val_width))
            .body(|mut body| {
                let pc_str = match self.mem_adr_base {
                    Base::Bin => format!("{pc:#b}"),
                    Base::Dec => format!("{pc}"),
                    Base::Hex => format!("{pc:#x}"),
                };
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("PC");
                    });
                    row.col(|ui| {
                        ui.label(pc_str);
                    });
                });
                for i in 0..8 {
                    let val = self.emu_registers[4 + i];
                    let val_str = match self.regs_base {
                        Base::Bin => format!("{val:#034b}"),
                        Base::Dec => format!("{val}"),
                        Base::Hex => format!("{val:#010x}"),
                    };
                    body.row(20.0, |mut row| {
                        row.col(|ui| match i {
                            6 => {
                                ui.label("SP");
                            }
                            7 => {
                                ui.label("FP");
                            }
                            _ => {
                                ui.label(format!("R{i}"));
                            }
                        });
                        row.col(|ui| {
                            ui.label(val_str);
                        });
                    });
                }
            });
        //ui.separator();
        if self.emu_halted {
            ui.label("HALT");
        }
    }

    fn ioview(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.label("=CRT");
        // =CRT
        Frame::side_top_panel(&ctx.style())
            .fill(Color32::BLACK)
            .show(ui, |ui| {
                ui.label(self.buf_out.as_str());
                ui.allocate_space(egui::vec2(ui.available_width(), 0.0))
            });

        ui.separator();

        ui.add_enabled_ui(self.emu_waiting_for_in, |ui| {
            ui.label(
                RichText::new("=KBD")
                    .font(FONT_TBL.clone())
                    .color(Color32::WHITE),
            );
            TextEdit::singleline(&mut self.buf_in)
                .hint_text("Type a number")
                .show(ui);
            if ui.button("Send").clicked() {
                if self.buf_in.parse::<i32>().is_ok() {
                    self.emu_tx
                        .send(CtrlMSG::In(self.buf_in.parse::<i32>().unwrap()));
                    self.buf_in = String::new();
                    self.emu_waiting_for_in = false;
                } else {
                    self.buf_in = "Invalid input!".to_owned();
                }
            }
        });
    }

    fn display(&mut self, ctx: &Context, ui: &mut Ui) {
        let target_w = clamp(ui.available_width(), 0., 800.);

        let w = target_w as u32;
        let h = (target_w * (120. / 160.)) as u32;
        self.emu_displaybuffer = Some(image::ImageBuffer::new(w, h));
        for (x, y, pixels) in self
            .emu_displaybuffer
            .as_mut()
            .unwrap()
            .enumerate_pixels_mut()
        {
            let px_x = (x as f32 / w as f32 * 160.) as u32;
            let px_y = (y as f32 / h as f32 * 120.) as u32;
            *pixels = image::Rgba([
                (self.emu_dispvec[(px_x + px_y * 160) as usize] >> 4) as u8,
                (self.emu_dispvec[(px_x + px_y * 160) as usize]) as u8,
                (self.emu_dispvec[(px_x + px_y * 160) as usize] << 4) as u8,
                255,
            ]);
        }
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &self.emu_displaybuffer.as_ref().unwrap(),
        );
        let render_result = RetainedImage::from_color_image("0.png", color_image);

        self.emu_displayimage = Some(render_result);
        if let Some(img) = &self.emu_displayimage {
            img.show(ui);
            /*if ui.button("save").clicked() {
                if let Some(buf) = &self.emu_displaybuffer {
                    println!("img saved!");
                    buf.save("test.png").unwrap();
                }
            };*/
            ui.separator();
        }
    }

    // Refresh cached regs and memory
    fn refresh_emu_state(&mut self) {
        self.emu_tx.send(CtrlMSG::GetState);
        self.emu_tx.send(CtrlMSG::GetRegs);
        self.emu_tx.send(CtrlMSG::GetMem(0..self.emu_memory_len));
    }

    fn stateview(&mut self, ui: &mut Ui) {
        ui.label("Emulation speed:");
        ui.label(format_with_decimals_in_range(self.emu_achieved_speed as f64, 1..=1) + "%");
    }
}
