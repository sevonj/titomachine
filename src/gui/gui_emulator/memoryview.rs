// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! This module houses the Memory Explorer GUI
//!

use std::{default::Default, ops::Range};
use std::collections::{HashMap, HashSet};
use egui::{CentralPanel, Color32, Frame, Image, include_image, RichText, ScrollArea, Sense, SidePanel, Slider, TopBottomPanel, Ui, scroll_area::ScrollBarVisibility};
use egui_extras::{Column, TableBody, TableBuilder, TableRow};
use num_traits::ToPrimitive;
use crate::gui::{Radix, View};
use crate::gui::gui_emulator::{COL_TEXT, COL_TEXT_HI, FONT_TBL, FONT_TBLH};
use crate::gui::gui_emulator::disassembler::disassemble_instruction;

const MEM_SIZE: usize = 0x2000;
const COLOR_SEGMENT_NONE: Color32 = Color32::from_rgb(60, 60, 60);
const COLOR_SEGMENT_CODE: Color32 = Color32::from_rgb(167, 115, 0);
const COLOR_SEGMENT_DATA: Color32 = Color32::from_rgb(046, 137, 133);
const COLOR_SEGMENT_STACK: Color32 = Color32::from_rgb(159, 075, 150);
const COLOR_BREAKPOINT: Color32 = Color32::from_rgb(239, 80, 57);
const COLOR_BREAKPOINT_OPTION: Color32 = Color32::from_rgb(228, 122, 119);

/// Which segment does an address belong to.
enum Segment {
    None,
    Code,
    Data,
    Stack,
}

/// MemoryView is the UI component responsible for the memory viewer panel.
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct MemoryView {
    /// First address to cache
    #[serde(skip)] view_cache_start: usize,
    /// Number of addresses to cache
    #[serde(skip)] view_cache_size: usize,
    /// Currently cached addresses
    #[serde(skip)] view_cache: HashMap<usize, i32>,

    /// Multiple symbols may exist for an address, because of _consts_
    #[serde(skip)] symbol_table: HashMap<usize, Vec<String>>,
    /// Comment for an address.
    #[serde(skip)] comment_table: HashMap<usize, String>,
    ///
    #[serde(skip)] breakpoints: HashSet<usize>,

    /// Is the emulated machine both turned on and not paused
    #[serde(skip)] pub is_playing: bool,

    /// Memory view displays where PC, SP, and FP point to
    #[serde(skip)] pub cpu_pc: usize,
    /// Memory view displays where PC, SP, and FP point to
    #[serde(skip)] pub cpu_sp: usize,
    /// Memory view displays where PC, SP, and FP point to
    #[serde(skip)] pub cpu_fp: usize,

    /// Code segment start address
    #[serde(skip)] pub start_code: usize,
    /// Data segment start address
    #[serde(skip)] pub start_data: usize,
    /// Stack start address
    #[serde(skip)] pub start_stack: usize,

    /// Toggle visibility.
    visible: bool,
    /// Memory view follows PC register while playing
    pub follow_pc: bool,

    /// Which base to show address in
    view_addr_base: Radix,
    /// Which base to show value in
    view_value_base: Radix,

    // Done: Display Symbols
    // Todo: Load symbol table
    // Todo: Comments
    // Done: Mouse scroll
    // Todo: Breakpoints
    // Done: Adjust to screen size
    // Done: Hide overflow
    // Done: Toggle follow PC
    // Done: Only follow PC if playing
    // Done: Set addr radix
    // Done: Set value radix
    // Done: remember radix
}

impl MemoryView {
    pub fn new() -> Self {
        MemoryView {
            view_cache_start: 0,
            view_cache_size: 32,
            view_cache: HashMap::new(),
            symbol_table: HashMap::new(),
            comment_table: HashMap::new(),
            breakpoints: HashSet::new(),
            is_playing: false,
            cpu_pc: 0,
            cpu_sp: 0,
            cpu_fp: 0,
            start_code: MEM_SIZE,
            start_data: MEM_SIZE,
            start_stack: MEM_SIZE,
            visible: true,
            follow_pc: true,
            view_addr_base: Radix::Dec,
            view_value_base: Radix::Dec,
        }
    }

    pub fn reset(&mut self) {
        self.view_cache_start = 0;
        self.symbol_table.clear();
        self.comment_table.clear();
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
        self.view_cache.clear();
        for (offset, value) in values.iter().enumerate() {
            let address = range_start + offset;
            self.view_cache.insert(address, value.to_owned());
        }
    }

    pub fn set_symbol_table(&mut self, table: HashMap<usize, Vec<String>>) {
        self.symbol_table = table;
    }

    pub fn set_comment_table(&mut self, table: HashMap<usize, String>) {
        self.comment_table = table;
    }

    fn get_segment_from_address(&self, address: usize) -> Segment {
        if address >= MEM_SIZE {
            Segment::None
        } else if address >= self.start_stack {
            Segment::Stack
        } else if address >= self.start_data {
            Segment::Data
        } else if address >= self.start_code {
            Segment::Code
        } else {
            Segment::None
        }
    }

    /// Determine how many table rows can fit on screen. May give one too many, but overflow should
    /// be hidden.
    fn get_table_rows_fit(&self, ui: &Ui) -> usize {
        let row_height = 23.0;
        (ui.available_height() / row_height) as usize
    }

    /// Add an address-entry-displaying row to the memory view table.
    fn add_table_row(&mut self, body: &mut TableBody, address: usize) {
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
                    self.add_table_address(&mut row, address, font_color);
                    self.add_table_value(&mut row, value, font_color);
                    self.add_table_disassembly(&mut row, value, font_color);
                    self.add_table_pointers(&mut row, address, font_color);
                });
            }

            // Display placeholder
            None => {
                body.row(20.0, |mut row| {
                    self.add_table_address(&mut row, address, font_color);
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
    fn add_table_address(&mut self, row: &mut TableRow, address: usize, font_color: Color32) {
        let text = self.view_addr_base.format_addr(address);
        let color = match self.get_segment_from_address(address) {
            Segment::None => COLOR_SEGMENT_NONE,
            Segment::Code => COLOR_SEGMENT_CODE,
            Segment::Data => COLOR_SEGMENT_DATA,
            Segment::Stack => COLOR_SEGMENT_STACK
        };
        row.col(|ui| {
            // Segment marker
            let _segmark = ui.add(Image::new(include_image!("../../assets/memview_segment_marker.png"))
                .fit_to_original_size(1.0).tint(color)
                .sense(Sense { click: true, drag: false, focusable: false })
            );

            // Breakpoints
            /*
            match self.breakpoints.contains(&address) {
                false => {
                    // Draw _add breakpoint_ hint
                    if segmark.hovered() {
                        ui.add(Image::new(include_image!("../../assets/memview_breakpoint.png"))
                            .fit_to_original_size(1.0).tint(COLOR_BREAKPOINT_OPTION)
                        );
                    }
                    // Add breakpoint
                    if segmark.clicked() {
                        self.breakpoints.insert(address);
                    }
                }
                true => if self.breakpoints.contains(&address) {
                    // Draw breakpoint
                    let bpmark = ui.add(Image::new(include_image!("../../assets/memview_breakpoint.png"))
                        .fit_to_original_size(1.0).tint(COLOR_BREAKPOINT)
                        .sense(Sense { click: true, drag: false, focusable: false })
                    );
                    // Remove breakpoint
                    if bpmark.clicked() || segmark.clicked() {
                        self.breakpoints.remove(&address);
                    };
                }
            } */

            // Address label
            ui.label(RichText::new(text)
                .font(FONT_TBL.clone())
                .color(font_color)
            );
        });
    }

    /// Table Shortcut: Value column
    fn add_table_value(&self, row: &mut TableRow, value: i32, font_color: Color32) {
        let text = self.view_value_base.format_i32(value.to_owned());
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

        if self.cpu_pc == address || self.cpu_sp == address || self.cpu_fp == address || symbols.is_some() {
            text += "<-- ";
        }
        if self.cpu_pc == address { text += "PC "; }
        if self.cpu_sp == address { text += "SP "; }
        if self.cpu_fp == address { text += "FP "; }

        if let Some(vec) = symbols {
            for symbol in vec {
                text += format!("{} ", symbol).as_str()
            }
        }

        row.col(|ui| { ui.label(RichText::new(text).font(FONT_TBL.clone()).color(font_color)); });
    }

    /// Table Shortcut: Header column
    fn add_table_heading(&self, header: &mut TableRow, title: &str) {
        header.col(|ui| {
            ui.heading(RichText::new(title).font(FONT_TBLH.clone()));
        });
    }

    fn scroll_to_pc(&mut self) {
        if self.cpu_pc > 4 {
            self.view_cache_start = self.cpu_pc - 4;
        } else {
            self.view_cache_start = 0
        }
    }
}

impl View for MemoryView {
    fn ui(&mut self, ui: &mut Ui) {
        // Memview titlebar
        TopBottomPanel::top("memview_titlebar")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.visible, "Memory Explorer").clicked() {
                        self.visible = !self.visible
                    }
                    if !self.visible {
                        return;
                    }
                    ui.menu_button("Options", |ui| {
                        ui.checkbox(&mut self.follow_pc, "Follow Program Counter");
                        ui.label("Display address as");
                        ui.radio_value(&mut self.view_addr_base, Radix::Bin, "Binary");
                        ui.radio_value(&mut self.view_addr_base, Radix::Dec, "Decimal");
                        ui.radio_value(&mut self.view_addr_base, Radix::Hex, "Hex");
                        ui.label("Display value as");
                        ui.radio_value(&mut self.view_value_base, Radix::Bin, "Binary");
                        ui.radio_value(&mut self.view_value_base, Radix::Dec, "Decimal");
                        ui.radio_value(&mut self.view_value_base, Radix::Hex, "Hex");
                    });
                    if ui.button("Find PC").clicked() {
                        self.scroll_to_pc();
                    }
                });
            });

        if !self.visible {
            return;
        }

        // Memview scrollbar
        SidePanel::right("memview_scroll")
            .resizable(false)
            .max_width(22.0)
            .frame(Frame::none())
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
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
            });

        // Memview main panel
        CentralPanel::default()
            .show_inside(ui, |ui| {
                let rows_to_display = self.get_table_rows_fit(ui);
                self.view_cache_size = rows_to_display;

                // Mouse scroll
                let scroll = -ui.input(|i| i.raw_scroll_delta).y.to_isize().unwrap().clamp(-2, 2);
                self.view_cache_start = self.view_cache_start.saturating_add_signed(scroll);

                // Follow PC
                if self.follow_pc && self.is_playing {
                    self.scroll_to_pc();
                }

                // The purpose of this non-interactive ScrollArea is to hide overflow.
                ScrollArea::vertical()
                    .enable_scrolling(false)
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
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
                                self.add_table_heading(&mut header, "Pointers");
                            })
                            .body(|mut body| {
                                for off in 0..=rows_to_display {
                                    self.add_table_row(&mut body, self.view_cache_start + off);
                                }
                            });
                    });
            });
    }
}