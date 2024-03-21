// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! This module contains the CPU View Panel
//!

use std::{default::Default, ops::Range};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use egui::{CentralPanel, Color32, Frame, Image, include_image, RichText, ScrollArea, Sense, SidePanel, Slider, TopBottomPanel, Ui, scroll_area::ScrollBarVisibility, TextBuffer};
use egui_extras::{Column, TableBody, TableBuilder, TableRow};
use libttktk::disassembler::disassemble_instruction;
use num_traits::ToPrimitive;
use crate::config::Config;
use crate::emulator::emu_debug::CtrlMSG;
use crate::gui::{Radix, EmulatorPanel};
use crate::gui::gui_emulator::{COL_TEXT, COL_TEXT_HI, FONT_TBL, FONT_TBLH};

/// CPUView is the UI component responsible for the memory viewer panel.
pub(crate) struct CPUView {
    pub cpu_halt: bool,
    pub cpu_cu_pc: i32,
    pub cpu_cu_sr: i32,
    pub cpu_gpr_r0: i32,
    pub cpu_gpr_r1: i32,
    pub cpu_gpr_r2: i32,
    pub cpu_gpr_r3: i32,
    pub cpu_gpr_r4: i32,
    pub cpu_gpr_r5: i32,
    pub cpu_gpr_sp: i32,
    pub cpu_gpr_fp: i32,
}

impl CPUView {
    pub fn new() -> Self {
        CPUView {
            cpu_halt: true,
            cpu_cu_pc: 0,
            cpu_cu_sr: 0,
            cpu_gpr_r0: 0,
            cpu_gpr_r1: 0,
            cpu_gpr_r2: 0,
            cpu_gpr_r3: 0,
            cpu_gpr_r4: 0,
            cpu_gpr_r5: 0,
            cpu_gpr_sp: 0,
            cpu_gpr_fp: 0,
        }
    }

    /// Table Shortcut: Program counter. PC display has one difference from GPRs: it formats the
    /// value in the same base as memory addresses.
    fn add_row_pc(&self, body: &mut TableBody, config: &mut Config) {
        println!("PC: {}", self.cpu_cu_pc);
        body.row(20.0, |mut row| {
            row.col(|ui| {
                let value_str = config.memview_addr_base.format_i32(self.cpu_cu_pc);
                ui.label(RichText::new(format!("PC {value_str}")).font(FONT_TBL.clone()));
            });
        });
    }

    /// Table Shortcut: General Purpose Registers
    fn add_row_gpr(&self, body: &mut TableBody, config: &mut Config, name: &str, value: i32) {
        body.row(20.0, |mut row| {
            row.col(|ui| {
                let value_str = config.cpuview_regs_base.format_i32(value);
                ui.label(RichText::new(format!("{name} {value_str}")).font(FONT_TBL.clone()));
            });
        });
    }

    /// Table Shorcut: Status Register
    fn add_row_sr(&self, body: &mut TableBody) {
        body.row(20.0, |mut row| {
            row.col(|ui| {
                ui.label(RichText::new("Status:").font(FONT_TBL.clone()));
                if self.cpu_halt {
                    ui.label(RichText::new("[HALT]").font(FONT_TBL.clone()));
                }
            });
        });
        body.row(20.0, |mut row| {
            row.col(|ui| {
                let value_str = format!(
                    "{}{}{}{}{}{}{}{}{}{}{}",
                    if self.cpu_cu_sr & (1 << 31) != 0 { "G" } else { "-" },
                    if self.cpu_cu_sr & (1 << 30) != 0 { "E" } else { "-" },
                    if self.cpu_cu_sr & (1 << 29) != 0 { "L" } else { "-" },
                    if self.cpu_cu_sr & (1 << 28) != 0 { "O" } else { "-" },
                    if self.cpu_cu_sr & (1 << 27) != 0 { "Z" } else { "-" },
                    if self.cpu_cu_sr & (1 << 26) != 0 { "U" } else { "-" },
                    if self.cpu_cu_sr & (1 << 25) != 0 { "M" } else { "-" },
                    if self.cpu_cu_sr & (1 << 24) != 0 { "I" } else { "-" },
                    if self.cpu_cu_sr & (1 << 23) != 0 { "S" } else { "-" },
                    if self.cpu_cu_sr & (1 << 22) != 0 { "P" } else { "-" },
                    if self.cpu_cu_sr & (1 << 21) != 0 { "D" } else { "-" },
                );
                ui.label(RichText::new(format!("{value_str}")).font(FONT_TBL.clone()));
            });
        });
    }

    /// Table Shortcut: Header column
    fn add_table_heading(&self, header: &mut TableRow, title: &str) {
        header.col(|ui| {
            ui.heading(RichText::new(title).font(FONT_TBLH.clone()));
        });
    }
}

impl EmulatorPanel for CPUView {
    fn ui(&mut self, ui: &mut Ui, config: &mut Config, sender: &Sender<CtrlMSG>) {

        // Memview titlebar

        ui.horizontal(|ui| {
            match config.cpuview_visible {
                true => if ui.selectable_label(config.cpuview_visible, "⏷ CPU").clicked() {
                    config.cpuview_visible = false
                }
                false => if ui.selectable_label(config.cpuview_visible, "⏵ CPU").clicked() {
                    config.cpuview_visible = true
                }
            };
            if !config.cpuview_visible {
                return;
            }
            ui.menu_button("Show as", |ui| {
                if ui.radio_value(&mut config.cpuview_regs_base, Radix::Bin, "Binary").clicked() { ui.close_menu(); };
                if ui.radio_value(&mut config.cpuview_regs_base, Radix::Dec, "Decimal").clicked() { ui.close_menu(); };
                if ui.radio_value(&mut config.cpuview_regs_base, Radix::Hex, "Hex").clicked() { ui.close_menu(); };
            });
        });

        ui.separator();

        if !config.cpuview_visible {
            return;
        }

        // Memview main panel
        TableBuilder::new(ui)
            .resizable(false)
            .striped(true)
            .vscroll(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(106.0))
            .body(|mut body| {
                self.add_row_pc(&mut body, config);
                self.add_row_gpr(&mut body, config, "R0", self.cpu_gpr_r0);
                self.add_row_gpr(&mut body, config, "R1", self.cpu_gpr_r1);
                self.add_row_gpr(&mut body, config, "R2", self.cpu_gpr_r2);
                self.add_row_gpr(&mut body, config, "R3", self.cpu_gpr_r3);
                self.add_row_gpr(&mut body, config, "R4", self.cpu_gpr_r4);
                self.add_row_gpr(&mut body, config, "R5", self.cpu_gpr_r5);
                self.add_row_gpr(&mut body, config, "SP", self.cpu_gpr_sp);
                self.add_row_gpr(&mut body, config, "FP", self.cpu_gpr_fp);
                self.add_row_sr(&mut body);
            });

        ui.separator();
    }
}