use self::gui_devices::GUIDevice;

use super::Radix;
use crate::{emulator::emu_debug::CtrlMSG, TitoApp, gui::EmulatorPanel};

pub(crate) mod gui_devices;
pub(crate) mod memoryview;

use eframe::emath::format_with_decimals_in_range;
use egui::{Button, Color32, Context, FontId, Frame, RichText, Ui};
use egui_extras::{Column, TableBuilder};

const FONT_TBL: FontId = FontId::monospace(12.0);
const FONT_TBLH: FontId = FontId::proportional(12.5);
#[allow(dead_code)]
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
            let _ = self.tx_ctrl.send(CtrlMSG::PlaybackPlayPause(false));
            if self.emu_running {
                self.stop_emulation();
            } else {
                self.emu_running = true;
                let _ = self.tx_ctrl.send(CtrlMSG::EnableBreakpoints(self.config.memview_breakpoints_enabled));
                let _ = self.tx_ctrl.send(CtrlMSG::PlaybackStart);
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
                let _ = self.tx_ctrl
                    .send(CtrlMSG::PlaybackPlayPause(self.emu_playing));
            }
            ui.add_enabled_ui(!self.emu_playing, |ui| {
                if ui
                    .add(Button::new(RichText::new("|▶")).min_size(egui::vec2(24.0, 0.0)))
                    .clicked()
                {
                    let _ = self.tx_ctrl.send(CtrlMSG::PlaybackTick);
                    if self.config.memview_follow_pc {
                        self.memoryview.jump_to_pc();
                    }
                    // Repaint because we touched memory viewer
                    ctx.request_repaint()
                }
            })
        });

        ui.separator();

        if ui.button("Reset").clicked() {
            let _ = self.tx_ctrl.send(CtrlMSG::Reset());
            self.dev_legacyio.reset();
            self.dev_display.reset();
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

    pub fn emulator_panel(&mut self, ctx: &Context, _: &mut Ui) {
        self.refresh_emu_state();

        egui::CentralPanel::default().show(ctx, |_| {
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
                    self.dev_legacyio.gui_panel(ctx, ui);
                    ui.separator();
                });
            egui::CentralPanel::default().show(ctx, |_ui| {
                // Display
                if self.emugui_display {
                    egui::TopBottomPanel::top("display")
                        .resizable(true)
                        .show(ctx, |ui| {
                            self.dev_display.gui_panel(ctx, ui);
                        });
                }
                // Memory View
                egui::CentralPanel::default()
                    .frame(Frame::none())
                    .show(ctx, |ui| {
                        self.memoryview.ui(ui, &mut self.config, &self.tx_ctrl);
                    });
            });
        });
    }

    fn regview(&mut self, ui: &mut Ui) {
        // CPU Registers
        ui.label("CPU Registers");
        let reg_name_width: f32 = 16.0;
        let reg_val_width: f32 = match self.regs_base == Radix::Bin {
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
                let pc_str = match self.config.memview_addr_base {
                    Radix::Bin => format!("{pc:#b}"),
                    Radix::Dec => format!("{pc}"),
                    Radix::Hex => format!("{pc:#x}"),
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
                        Radix::Bin => format!("{val:#034b}"),
                        Radix::Dec => format!("{val}"),
                        Radix::Hex => format!("{val:#010x}"),
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
            });
        let sr_str = format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
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
        ui.label(format!("Status:"));
        ui.label(RichText::new(sr_str).font(FONT_TBL.clone()));
        ui.label(format!("{}", if self.emu_halted { "HALT!" } else { "" }));
    }

    // Refresh cached regs and memory
    fn refresh_emu_state(&mut self) {
        let _ = self.tx_ctrl.send(CtrlMSG::GetState);
        let _ = self.tx_ctrl.send(CtrlMSG::GetMem(self.memoryview.get_view_cache_range()));
    }

    fn stateview(&mut self, ui: &mut Ui) {
        ui.label("Emulation speed:");
        ui.label(format_with_decimals_in_range(self.emu_achieved_speed as f64, 1..=1) + "%");
    }
}
