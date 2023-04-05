use super::Base;
use crate::{emulator::emu_debug::CtrlMSG, TitoApp};
pub mod instruction_parser;
use eframe::emath::format_with_decimals_in_range;
use egui::{Button, Color32, Context, FontId, Frame, Layout, RichText, TextEdit, Ui};
use egui_extras::{Column, RetainedImage, TableBuilder};
use instruction_parser::*;
use num_traits::{clamp, ToPrimitive};

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
            self.tx_ctrl.send(CtrlMSG::PlaybackPlayPause(false));
            if self.emu_running {
                self.stop_emulation();
            } else {
                self.emu_running = true;
                self.tx_ctrl.send(CtrlMSG::PlaybackStart);
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
                self.tx_ctrl
                    .send(CtrlMSG::PlaybackPlayPause(self.emu_playing));
            }
            ui.add_enabled_ui(!self.emu_playing, |ui| {
                if ui
                    .add(Button::new(RichText::new("|▶")).min_size(egui::vec2(24.0, 0.0)))
                    .clicked()
                {
                    self.tx_ctrl.send(CtrlMSG::PlaybackTick);
                }
            })
        });

        ui.separator();

        if ui.button("Reload").clicked() {
            self.tx_ctrl
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
                    egui::TopBottomPanel::top("display")
                        .resizable(true)
                        .show(ctx, |ui| {
                            self.display(ctx, ui);
                            self.tx_ctrl.send(CtrlMSG::GetDisp);
                        });
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

        egui::CentralPanel::default().show(ctx, |ui| {
            /*
             * Memview gives an illusion of scrolling through one large table that contains all
             * addresses, but it's size is actually always exactly what's visible on the screen.
             *
             * To keep the table always in view of the scroll area, we allocate the last known
             * scroll position worth of space before it.
             *
             * After adding the table, we allocate more space again to make the scrollarea total
             * height what the table would take if it contained every address.
             *
             * Table start offset is calculated from scroll position.
             */
            let row_height = 23.;
            let height = ui.available_height() - 30.;
            self.gui_memview_len = (height / row_height).to_i32().unwrap();
            let total_height = row_height * (self.emu_mem_len + 2) as f32 + 30.; // +2 because for some reason it fell short by that amount
            let view_height = height + 30.;
            self.gui_memview_scroll = egui::ScrollArea::vertical()
                .show(ui, |ui| {
                    ui.allocate_space(egui::Vec2 {
                        x: 0.,
                        y: self.gui_memview_scroll,
                    });
                    TableBuilder::new(ui)
                        .striped(true)
                        .auto_shrink([false; 2])
                        .max_scroll_height(f32::INFINITY)
                        .vscroll(false)
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
                            //let rowcount = self.emu_memory_len;
                            for i in 0..self.gui_memview_len {
                                if i >= self.gui_memview.len() as i32 {
                                    break;
                                }
                                let adr = self.gui_memview_off + i;
                                let val: i32 = self.gui_memview[i as usize];
                                let pc = self.emu_regs.pc;
                                let sp = self.emu_regs.gpr[6];
                                let fp = self.emu_regs.gpr[7];
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
                                let col = if adr == pc { COL_TEXT_HI } else { COL_TEXT };
                                body.row(20.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(
                                            RichText::new(adr_str)
                                                .font(FONT_TBL.clone())
                                                .color(col),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.label(
                                            RichText::new(val_str)
                                                .font(FONT_TBL.clone())
                                                .color(col),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.label(
                                            RichText::new(ins_str)
                                                .font(FONT_TBL.clone())
                                                .color(col),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.label(
                                            RichText::new(reg_str)
                                                .font(FONT_TBL.clone())
                                                .color(col),
                                        );
                                    });
                                });
                            }
                        });

                    ui.allocate_space(egui::Vec2 {
                        x: 0.,
                        y: total_height - self.gui_memview_scroll - view_height,
                    });
                    if self.emugui_follow_pc && self.emu_playing {
                        let pc_pos = row_height * self.emu_regs.pc as f32;
                        ui.scroll_to_rect(
                            egui::Rect {
                                min: egui::Pos2 {
                                    x: 0.,
                                    y: pc_pos - self.gui_memview_scroll,
                                },
                                max: egui::Pos2 {
                                    x: 0.,
                                    y: pc_pos - self.gui_memview_scroll + view_height,
                                },
                            },
                            Some(egui::Align::Center),
                        );
                    }
                })
                .state
                .offset
                .y;
            self.gui_memview_off = (self.gui_memview_scroll / row_height) as i32;
        });
        //    });
        //});
    }

    fn regview(&mut self, ui: &mut Ui) {
        // CPU Registers
        ui.label("CPU Registers");
        let reg_name_width: f32 = 16.0;
        let reg_val_width: f32 = match self.regs_base == Base::Bin {
            true => 256.0,
            false => 72.0,
        };

        let pc = self.emu_regs.pc;
        let sr = self.emu_regs.sr;
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
                    let val = self.emu_regs.gpr[i];
                    let val_str = match self.regs_base {
                        Base::Bin => format!("{val:#034b}"),
                        Base::Dec => format!("{val}"),
                        Base::Hex => format!("{val:#010x}"),
                    };
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            let regname;
                            match i {
                                6 => regname = "SP".into(),
                                7 => regname = "FP".into(),
                                _ => regname = format!("R{i}"),
                            };
                            ui.label(regname);
                        });
                        row.col(|ui| {
                            ui.label(val_str);
                        });
                    });
                }
                let sr_str = format!(
                    "{}{}{}\n{}{}{}{}{}{}{}{}",
                    if sr & (1 << 31) != 0 { "G" } else { "-" },
                    if sr & (1 << 30) != 0 { "E" } else { "-" },
                    if sr & (1 << 29) != 0 { "L" } else { "-" },
                    if sr & (1 << 28) != 0 { "O" } else { "-" },
                    if sr & (1 << 27) != 0 { "Z" } else { "-" },
                    if sr & (1 << 26) != 0 { "U" } else { "-" },
                    if sr & (1 << 25) != 0 { "M" } else { "-" },
                    if sr & (1 << 24) != 0 { "I" } else { "-" },
                    if sr & (1 << 23) != 0 { "S" } else { "-" },
                    if sr & (1 << 22) != 0 { "P" } else { "-" },
                    if sr & (1 << 21) != 0 { "D" } else { "-" },
                );
                body.row(40.0, |mut row| {
                    row.col(|ui| {
                        ui.label(format!(
                            "Status:\n{}",
                            if self.emu_halted { "HALT!" } else { "" }
                        ));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(sr_str).font(FONT_TBL.clone()));
                    });
                });
            });
    }

    fn ioview(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.label("=CRT");
        match self.rx_devcrt.try_recv() {
            Ok(n) => self.devcrt_out(n),
            Err(_) => (),
        }
        Frame::side_top_panel(&ctx.style())
            .fill(Color32::BLACK)
            .show(ui, |ui| {
                ui.label(self.buf_out.as_str());
                ui.allocate_space(egui::vec2(ui.available_width(), 0.0))
            });

        ui.separator();

        match self.rx_devkbdreq.try_recv() {
            Ok(_) => self.emu_waiting_for_in = true,
            Err(_) => (),
        }
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
                    self.tx_devkbd.send(self.buf_in.parse::<i32>().unwrap());
                    self.buf_in = String::new();
                    self.emu_waiting_for_in = false;
                } else {
                    self.buf_in = "Invalid input!".to_owned();
                }
            }
        });
    }

    fn display(&mut self, ctx: &Context, ui: &mut Ui) {
        // Determine image size based on available w / h, whichever fits a smaller image
        let target_h = clamp(ui.available_height(), 120., 400.); // size limited for performance
        let target_w = clamp(ui.available_width(), 160., f32::INFINITY);
        let w;
        let h;
        if target_w > target_h * (160. / 120.) {
            w = (target_h * (160. / 120.)) as u32;
            h = target_h as u32;
        } else {
            w = target_w as u32;
            h = (target_w * (120. / 160.)) as u32;
        }
        ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            self.emu_displaybuffer = Some(image::ImageBuffer::new(w, h));
            // This is a terribly inefficient way to make the image
            // TODO: figure out how to just rescale the original res pic.
            for (x, y, pixels) in self
                .emu_displaybuffer
                .as_mut()
                .unwrap()
                .enumerate_pixels_mut()
            {
                // px_off = px_x + px_y * 160
                let px_off = (x * 160 / w) + (y * 120 / h) * 160;
                *pixels = self.framebuffer[px_off as usize];
            }
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                &self.emu_displaybuffer.as_ref().unwrap(),
            );
            let render_result = RetainedImage::from_color_image("0.png", color_image);
            self.emu_displayimage = Some(render_result);
            if let Some(img) = &self.emu_displayimage {
                img.show(ui);
            }
        });
    }

    // Refresh cached regs and memory
    fn refresh_emu_state(&mut self) {
        self.tx_ctrl.send(CtrlMSG::GetState);
        self.tx_ctrl.send(CtrlMSG::GetRegs);
        self.tx_ctrl.send(CtrlMSG::GetMem(
            self.gui_memview_off..self.gui_memview_off + self.gui_memview_len,
        ));
    }

    fn stateview(&mut self, ui: &mut Ui) {
        ui.label("Emulation speed:");
        ui.label(format_with_decimals_in_range(self.emu_achieved_speed as f64, 1..=1) + "%");
    }
}
