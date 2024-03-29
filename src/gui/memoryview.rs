// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! This module houses the Memory Explorer GUI
//!

use egui::Button;
use std::{default::Default, ops::Range};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use egui::{CentralPanel, Color32, Frame, Image, include_image, RichText, ScrollArea, Sense, SidePanel, Slider, TopBottomPanel, Ui, scroll_area::ScrollBarVisibility};
use egui_extras::{Column, TableBody, TableBuilder, TableRow};
use libttktk::disassembler::disassemble_instruction;
use num_traits::ToPrimitive;
use crate::config::Config;
use crate::emulator::emu_debug::CtrlMSG;
use crate::gui::{Radix, EmulatorPanel};
use crate::gui::{COL_TEXT, COL_TEXT_HI, FONT_TBL, FONT_TBLH};


const MEM_SIZE: usize = 0x2000;
const COLOR_SEGMENT_NONE: Color32 = Color32::from_rgb(60, 60, 60);
const COLOR_SEGMENT_CODE: Color32 = Color32::from_rgb(167, 115, 0);
const COLOR_SEGMENT_DATA: Color32 = Color32::from_rgb(046, 137, 133);
const COLOR_SEGMENT_STACK: Color32 = Color32::from_rgb(159, 075, 150);
const COLOR_BREAKPOINT: Color32 = Color32::from_rgb(239, 80, 57);
const COLOR_BREAKPOINT_OPTION: Color32 = Color32::from_rgb(228, 122, 119);
const COLOR_BREAKPOINT_DISABLED: Color32 = Color32::from_additive_luminance(0x1f);

/// Which segment does an address belong to.
enum MemorySegment {
    None,
    Code,
    Data,
    Stack,
}

/// MemoryView is the UI component responsible for the memory viewer panel.
pub(crate) struct MemoryView {
    /// First address to cache
    view_cache_start: usize,
    /// Number of addresses to cache
    view_cache_size: usize,
    /// Currently cached addresses
    view_cache: HashMap<usize, i32>,

    /// Multiple symbols may exist for an address, because of _consts_
    symbol_table: HashMap<usize, Vec<String>>,
    /// Set of addresses that contains a breakpoint.
    breakpoints: HashSet<usize>,

    /// Is the emulated machine both turned on and not paused
    pub is_playing: bool,

    /// Memory view displays where PC, SP, and FP point to
    pub cpu_pc: usize,
    /// Memory view displays where PC, SP, and FP point to
    pub cpu_sp: usize,
    /// Memory view displays where PC, SP, and FP point to
    pub cpu_fp: usize,

    /// Code segment start address
    pub start_code: usize,
    /// Data segment start address
    pub start_data: usize,
    /// Stack start address
    pub start_stack: usize,
}

impl MemoryView {
    pub fn new() -> Self {
        MemoryView {
            view_cache_start: 0,
            view_cache_size: 32,
            view_cache: HashMap::new(),
            symbol_table: HashMap::new(),
            breakpoints: HashSet::new(),
            is_playing: false,
            cpu_pc: 0,
            cpu_sp: 0,
            cpu_fp: 0,
            start_code: MEM_SIZE,
            start_data: MEM_SIZE,
            start_stack: MEM_SIZE,
        }
    }

    /// Reset needs to be called when loading a new program. Otherwise, old stuff like breakpoints
    /// may linger in the memory view despite being removed from the emulator.
    pub fn reset(&mut self) {
        self.view_cache_start = 0;
        self.symbol_table.clear();
        self.breakpoints.clear();
        self.start_code = MEM_SIZE;
        self.start_data = MEM_SIZE;
        self.start_stack = MEM_SIZE;
    }

    /// Range of addresses currently visible on screen
    pub fn get_view_cache_range(&self) -> Range<u32> {
        (self.view_cache_start as u32)..(self.view_cache_start + self.view_cache_size + 1) as u32
    }
    /// First address currently visible on screen
    pub fn get_view_cache_start(&self) -> usize {
        self.view_cache_start
    }

    /// Update memoryview's cached addresses. range_start tells the first address.
    /// Use get_view_cache_range() to know which addresses memoryview wants.
    pub fn set_view_cache(&mut self, range_start: usize, values: Vec<i32>) {
        //self.view_cache.clear();
        for (offset, value) in values.iter().enumerate() {
            let address = range_start + offset;
            self.view_cache.insert(address, value.to_owned());
        }
    }

    /// Give memoryview a copy of B91 symbol table.
    pub fn set_symbol_table(&mut self, table: HashMap<String, i32>) {
        self.symbol_table.clear();
        for (name, value) in table {
            // Negative addresses don't exist, they wouldn't show up anyway.
            if value < 0 {
                continue;
            }
            let addr = value as usize;
            match self.symbol_table.get_mut(&addr) {
                None => {
                    self.symbol_table.insert(addr, vec![name]);
                }
                Some(vec) => {
                    vec.push(name);
                }
            }
        }
    }

    /// Scroll view to PC location
    pub fn jump_to_pc(&mut self) {
        if self.cpu_pc > 4 {
            self.view_cache_start = self.cpu_pc - 4;
        } else {
            self.view_cache_start = 0
        }
    }

    /// Which segment does an address belong?
    fn get_segment_from_address(&self, address: usize) -> MemorySegment {
        if address >= MEM_SIZE {
            MemorySegment::None
        } else if address >= self.start_stack {
            MemorySegment::Stack
        } else if address >= self.start_data {
            MemorySegment::Data
        } else if address >= self.start_code {
            MemorySegment::Code
        } else {
            MemorySegment::None
        }
    }

    /// Determine how many table rows can fit on screen. May give one too many, but overflow should
    /// be hidden.
    fn get_table_rows_fit(&self, ui: &Ui) -> usize {
        let row_height = 23.0;
        (ui.available_height() / row_height) as usize
    }

    /// Add an address-entry-displaying row to the memory view table.
    fn add_table_row(
        &mut self,
        config: &mut Config,
        sender: &Sender<CtrlMSG>,
        body: &mut TableBody,
        address: usize,
    ) {
        // Out of bounds
        if address >= MEM_SIZE {
            return;
        }
        let font_color = match self.cpu_pc == address {
            true => COL_TEXT_HI,
            false => COL_TEXT
        };

        match self.view_cache.get(&address) {

            // Display entry
            Some(value) => {
                let value = value.to_owned();
                body.row(20.0, |mut row| {
                    self.add_table_address(config, sender, &mut row, address, font_color);
                    self.add_table_value(config, &mut row, value, font_color);
                    self.add_table_disassembly(&mut row, value, font_color);
                    self.add_table_pointers(&mut row, address, font_color);
                });
            }

            // Display placeholder
            None => {
                body.row(20.0, |mut row| {
                    self.add_table_address(config, sender, &mut row, address, font_color);
                    self.add_table_label(&mut row, "Fetching...", font_color);
                    self.add_table_label(&mut row, "", font_color);
                    self.add_table_label(&mut row, "", font_color);
                });
            }
        }
    }

    /// Table Shortcut: Generic label
    fn add_table_label(&self, row: &mut TableRow, text: &str, font_color: Color32) {
        row.col(|ui| {
            ui.label(RichText::new(text).font(FONT_TBL.clone()).color(font_color));
        });
    }

    /// Table Shortcut: Address column
    fn add_table_address(
        &mut self,
        config: &Config,
        sender: &Sender<CtrlMSG>,
        row: &mut TableRow,
        address: usize,
        font_color: Color32) {
        let text = config.memview_addr_base.format_addr(address);
        let color = match self.get_segment_from_address(address) {
            MemorySegment::None => COLOR_SEGMENT_NONE,
            MemorySegment::Code => COLOR_SEGMENT_CODE,
            MemorySegment::Data => COLOR_SEGMENT_DATA,
            MemorySegment::Stack => COLOR_SEGMENT_STACK
        };
        row.col(|ui| {
            // Segment marker
            ui.add(Image::new(include_image!("../assets/memview_segment_marker.png"))
                .fit_to_original_size(1.0).tint(color)
            );

            // Address label
            let addr_label = ui.label(RichText::new(text)
                .font(FONT_TBL.clone())
                .color(font_color)
            );

            // Breakpoints
            let bp_color = if self.breakpoints.contains(&address) {
                match config.memview_breakpoints_enabled {
                    true => COLOR_BREAKPOINT,
                    false => COLOR_BREAKPOINT_DISABLED
                }
            } else if addr_label.hovered() {
                COLOR_BREAKPOINT_OPTION
            } else {
                Color32::TRANSPARENT
            };

            let bpmark = ui.add(Image::new(include_image!("../assets/memview_breakpoint.png"))
                .fit_to_original_size(1.0).tint(bp_color)
                .sense(Sense { click: true, drag: false, focusable: false })
            );

            match self.breakpoints.contains(&address) {
                false => if addr_label.clicked() {
                    self.breakpoints.insert(address);
                    sender.send(CtrlMSG::InsertBreakpoint(address)).unwrap()
                }
                true => if bpmark.clicked() || addr_label.clicked() {
                    self.breakpoints.remove(&address);
                    sender.send(CtrlMSG::RemoveBreakpoint(address)).unwrap()
                }
            }
        });
    }

    /// Table Shortcut: Value column
    fn add_table_value(&self, config: &mut Config, row: &mut TableRow, value: i32, font_color: Color32) {
        let text = config.memview_value_base.format_i32(value.to_owned());
        row.col(|ui| {
            ui.label(RichText::new(text).font(FONT_TBL.clone()).color(font_color));
        });
    }

    /// Table Shortcut: Disassembly column
    fn add_table_disassembly(&self, row: &mut TableRow, value: i32, font_color: Color32) {
        let binding = disassemble_instruction(value.clone());
        let text = binding.as_str();
        row.col(|ui| {
            ui.label(RichText::new(text).font(FONT_TBL.clone()).color(font_color));
        });
    }

    /// Add a column that shows if PC, SP, FP, or any symbols point to this address
    fn add_table_pointers(&self, row: &mut TableRow, address: usize, font_color: Color32) {
        let mut text = String::new();
        let symbols = self.symbol_table.get(&address);

        if self.cpu_pc == address { text += "PC "; }
        if self.cpu_sp == address { text += "SP "; }
        if self.cpu_fp == address { text += "FP "; }

        if let Some(vec) = symbols {
            for symbol in vec {
                text += format!("{} ", symbol).as_str()
            }
        }

        row.col(|ui| {
            if self.cpu_pc == address || self.cpu_sp == address || self.cpu_fp == address || symbols.is_some() {
                // TODO: Make overlap
                ui.add(Image::new(include_image!("../assets/memview_pointer_arrow.png"))
                    .fit_to_original_size(1.0)
                    .tint(Color32::from_rgba_unmultiplied(255, 255, 255, 2))
                );
            }
            ui.label(RichText::new(text).font(FONT_TBL.clone()).color(font_color));
        });
    }

    /// Table Shortcut: Header column
    fn add_table_heading(&self, header: &mut TableRow, title: &str) {
        header.col(|ui| {
            ui.heading(RichText::new(title).font(FONT_TBLH.clone()));
        });
    }
}

impl EmulatorPanel for MemoryView {
    fn ui(&mut self, ui: &mut Ui, config: &mut Config, sender: &Sender<CtrlMSG>) {
        // Memview titlebar
        TopBottomPanel::top("memview_titlebar")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let toggle_text = if config.memview_visible { "⏷ Memory Explorer" } else { "⏵ Memory Explorer" };
                    if ui.add(Button::new(toggle_text).frame(false)).clicked(){
                        config.memview_visible = !config.memview_visible;
                    }
                    if !config.memview_visible {
                        return;
                    }
                    ui.menu_button("Options", |ui| {
                        if ui.checkbox(&mut config.memview_follow_pc, "Follow PC").clicked() { ui.close_menu(); };
                        ui.label("Display address as");
                        if ui.radio_value(&mut config.memview_addr_base, Radix::Bin, "Binary").clicked() { ui.close_menu(); };
                        if ui.radio_value(&mut config.memview_addr_base, Radix::Dec, "Decimal").clicked() { ui.close_menu(); };
                        if ui.radio_value(&mut config.memview_addr_base, Radix::Hex, "Hex").clicked() { ui.close_menu(); };
                        ui.label("Display value as");
                        if ui.radio_value(&mut config.memview_value_base, Radix::Bin, "Binary").clicked() { ui.close_menu(); };
                        if ui.radio_value(&mut config.memview_value_base, Radix::Dec, "Decimal").clicked() { ui.close_menu(); };
                        if ui.radio_value(&mut config.memview_value_base, Radix::Hex, "Hex").clicked() { ui.close_menu(); };
                        ui.label("Breakpoints");
                        if ui.checkbox(&mut config.memview_breakpoints_enabled, "Enabled").clicked() {
                            let _ = sender.send(CtrlMSG::EnableBreakpoints(config.memview_breakpoints_enabled));
                            ui.close_menu();
                        }
                        if ui.button("Clear all").clicked() {
                            self.breakpoints.clear();
                            let _ = sender.send(CtrlMSG::ClearBreakpoints);
                            ui.close_menu();
                        }
                    });
                    if ui.button("Go to PC").clicked() {
                        self.jump_to_pc();
                    }
                });
            });

        if !config.memview_visible {
            return;
        }

        // Memview scrollbar
        SidePanel::right("memview_scroll")
            .resizable(false)
            .max_width(22.0)
            .frame(Frame::none())
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                // Space accounts for table header row
                ui.add_space(20.);
                ui.spacing_mut().slider_width = ui.available_height();
                ui.add(Slider::new(&mut self.view_cache_start, 0x1fff..=0)
                    .vertical()
                    .smart_aim(false)
                    .show_value(false)
                    .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 1.6 })
                );
            });

        // Memview main panel
        CentralPanel::default()
            .show_inside(ui, |ui| {
                let rows_to_display = self.get_table_rows_fit(ui);
                self.view_cache_size = rows_to_display;

                // Mouse scroll
                let scroll = -ui.input(|i| i.raw_scroll_delta).y.to_isize().unwrap().clamp(-1, 1);
                self.view_cache_start = self.view_cache_start.saturating_add_signed(scroll);

                // Follow PC
                if config.memview_follow_pc & &self.is_playing {
                    self.jump_to_pc();
                }

                // The purpose of this non-interactive ScrollArea is to hide overflow.
                ScrollArea::vertical()
                    .enable_scrolling(false)
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ScrollArea::horizontal()
                            .show(ui, |ui| {

                                // The actual memory View Table
                                TableBuilder::new(ui)
                                    .resizable(false)
                                    .striped(true)
                                    .vscroll(false)
                                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                    .column(Column::auto()) // Address
                                    .column(Column::auto().at_least(78.0)) // Value
                                    .column(Column::exact(144.0)) // Disassembly
                                    .column(Column::remainder())// Pointers
                                    .header(20.0, |mut header| {
                                        self.add_table_heading(&mut header, "Addr");
                                        self.add_table_heading(&mut header, "Value");
                                        self.add_table_heading(&mut header, "Disassembly");
                                        self.add_table_heading(&mut header, "");
                                    })
                                    .body(|mut body| {
                                        for off in 0..=rows_to_display {
                                            self.add_table_row(config, sender, &mut body, self.view_cache_start + off);
                                        }
                                    });
                            });
                    });
            });
    }
}