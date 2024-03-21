// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! This provides emulator portion of the toolbar.
//!

use eframe::epaint::Color32;
use egui::{Button, RichText, Ui};
use crate::emulator::emu_debug::CtrlMSG;
use crate::TitoApp;

impl TitoApp {
    pub fn emulator_toolbar(&mut self, ui: &mut Ui) {
        // Power button
        let text_onoff = match self.emu_running {
            true => RichText::new("⏼on/off").color(Color32::WHITE),
            false => RichText::new("⏼on/off"),
        };
        if ui.selectable_label(self.emu_running, text_onoff).clicked() {
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
            // PlayPause Button
            let text_play;
            match self.emu_playing {
                true => text_play = RichText::new("⏸").color(Color32::WHITE),
                false => text_play = RichText::new("▶"),
            }
            if ui.add(Button::new(text_play).min_size(egui::vec2(24.0, 0.0))).clicked() {
                self.emu_playing = !self.emu_playing;
                let _ = self.tx_ctrl
                    .send(CtrlMSG::PlaybackPlayPause(self.emu_playing));
            }
            // Step Button
            ui.add_enabled_ui(!self.emu_playing, |ui| {
                if ui.add(Button::new(RichText::new("|▶")).min_size(egui::vec2(24.0, 0.0))).clicked() {
                    let _ = self.tx_ctrl.send(CtrlMSG::PlaybackTick);
                    if self.config.memview_follow_pc {
                        self.memoryview.jump_to_pc();
                    }
                }
            })
        });

        ui.separator();

        if ui.button("Reset").clicked() {
            let _ = self.tx_ctrl.send(CtrlMSG::Reset());
            self.legacytermview.clear();
            self.graphicsview.clear();
        }
        ui.separator();
    }
}