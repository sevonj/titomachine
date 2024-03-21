// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! This module contains the Legacy Terminal Panel: =CRT and =KBD
//!

use std::sync::mpsc::{Receiver, Sender};
use egui::{Color32, Frame, RichText, Ui, TextEdit, Stroke, TopBottomPanel, Button};
use crate::config::Config;
use crate::emulator::emu_debug::CtrlMSG;
use crate::gui::EmulatorPanel;

// The space at the end prevents buf_crt.lines() from dropping the last line.
const CRT_CLEAR_TEXT: &str = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n ";
const COLOR_INPUTHIGHLIGHT: Color32 = Color32::YELLOW;

/// LegacyTermView is the UI component responsible for the memory viewer panel.
pub(crate) struct LegacyTermView {
    rx_crt: Receiver<i32>,
    tx_kbd: Sender<i32>,
    rx_kbdreq: Receiver<()>,
    buf_kbd: String,
    buf_crt: String,

    waiting_for_input: bool,
}

impl LegacyTermView {
    pub fn new(rx_crt: Receiver<i32>, tx_kbd: Sender<i32>, rx_kbdreq: Receiver<()>) -> Self {
        LegacyTermView {
            rx_crt,
            tx_kbd,
            rx_kbdreq,
            buf_kbd: String::new(),
            buf_crt: CRT_CLEAR_TEXT.to_owned(),
            waiting_for_input: false,
        }
    }

    /// Add a value to CRT buffer
    pub fn crt_out(&mut self, n: i32) {
        // Remove top line
        self.buf_crt = self.buf_crt.lines().skip(1).map(|s| s.to_string() + "\n").collect();
        // Insert line to the end
        self.buf_crt += n.to_string().as_str();
    }

    pub fn clear(&mut self) {
        self.buf_kbd = String::new();
        self.buf_crt = CRT_CLEAR_TEXT.to_owned();
        self.unjam_input_wait();
    }

    /// If the emulator thread is waiting for input, it's frozen until it receives something.
    /// This will free the emulator by sending a dummy value.
    pub fn unjam_input_wait(&mut self) {
        if !self.waiting_for_input {
            return;
        }
        let _ = self.tx_kbd.send(0);
    }
}

impl EmulatorPanel for LegacyTermView {
    fn ui(&mut self, ui: &mut Ui, config: &mut Config, _sender: &Sender<CtrlMSG>) {
        // Update
        if let Ok(n) = self.rx_crt.try_recv() {
            self.crt_out(n)
        }
        if let Ok(_) = self.rx_kbdreq.try_recv() {
            self.waiting_for_input = true;
            // Pop the panel open if it's needed!
            config.legacyterm_visible = true;
        }

        // LegacyTerm titlebar
        TopBottomPanel::top("legacyterm_titlebar")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let toggle_text = if config.legacyterm_visible { "⏷ Legacy Terminal" } else { "⏵ Legacy Terminal" };
                    if ui.add(Button::new(toggle_text).frame(false)).clicked() {
                        config.legacyterm_visible = !config.legacyterm_visible;
                    }
                    if !config.legacyterm_visible {
                        return;
                    }
                });
            });

        if !config.legacyterm_visible {
            return;
        }

        TopBottomPanel::top("legacyterm_main")
            .resizable(false)
            .show_inside(ui, |ui| {
                // =CRT
                ui.label("=CRT");
                Frame::none()
                    .fill(Color32::BLACK)
                    .show(ui, |ui| {
                        ui.label(self.buf_crt.as_str());
                        ui.allocate_space(egui::vec2(ui.available_width(), 0.0))
                    });
                ui.separator();

                // =KBD
                ui.add_enabled_ui(self.waiting_for_input, |ui| {
                    // KBD Label
                    if self.waiting_for_input {
                        Frame::none()
                            .stroke(Stroke { width: 1.0, color: COLOR_INPUTHIGHLIGHT })
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("=KBD")
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                            });
                    } else {
                        ui.label(
                            RichText::new("=KBD")
                                .strong()
                                .color(Color32::WHITE),
                        );
                    }
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        // KBD text field
                        TextEdit::singleline(&mut self.buf_kbd).hint_text("Type a value").desired_width(78.0).show(ui);
                        // KBD send button
                        if ui.button("⬈").clicked() {
                            if self.buf_kbd.parse::<i32>().is_ok() {
                                let _ = self.tx_kbd.send(self.buf_kbd.parse::<i32>().unwrap());
                                self.buf_kbd = String::new();
                                self.waiting_for_input = false;
                            } else {
                                self.buf_kbd = "Invalid input!".to_owned();
                            }
                        }
                    });
                    ui.add_space(4.0);
                });
            });
    }
}