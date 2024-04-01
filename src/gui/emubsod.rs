// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! Emu-BSOD: Ideally no user will ever see this. Run tab will show this if emu thread crashes.
//!

use egui::{CentralPanel, FontFamily, RichText};
use egui::Color32;
use egui::Context;
use egui::Frame;

const COLOR_BG: Color32 = Color32::from_rgb(0x39, 0x3C, 0x51);
const COLOR_FG: Color32 = Color32::WHITE;

/// EmuBSODView is the GUI panel for registers.
pub(crate) struct EmuBSODView {}

impl EmuBSODView {
    pub fn new() -> Self {
        EmuBSODView {}
    }
    pub(crate) fn show(&mut self, ctx: &Context) {
        let frame = Frame::none()
            .fill(COLOR_BG);

        CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(128.0);
                    ui.label(
                        RichText::from(" titomachine \n")
                            .background_color(COLOR_FG)
                            .color(COLOR_BG)
                            .monospace()
                    );
                    ui.label(
                        RichText::from(
                            "Emulator thread has crashed. You can still save your work in the editor."
                        )
                            .color(COLOR_FG)
                            .monospace()
                    );
                });
            });
    }
}
