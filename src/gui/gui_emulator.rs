use self::gui_devices::GUIDevice;

use super::Radix;
use crate::{emulator::emu_debug::CtrlMSG, TitoApp, gui::EmulatorPanel};

pub(crate) mod gui_devices;
pub(crate) mod memoryview;
pub(crate) mod cpuview;
pub(crate) mod graphicsview;

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
            self.graphicsview.clear();
        }

        ui.separator();
        if ui
            .selectable_label(self.config.display_visible, "Show Graphics")
            .clicked()
        {
            self.config.display_visible = !self.config.display_visible;
        }
        ui.separator();
    }

    pub fn emulator_panel(&mut self, ctx: &Context, _: &mut Ui) {
        self.refresh_emu_state();

        egui::CentralPanel::default().show(ctx, |_| {

            // Status Panel
            egui::SidePanel::right("register_panel")
                .resizable(false)
                // Limit max width when base isn't binary, because auto shrink doesn't work properly
                // when separators and some other things are present.
                .max_width(if self.config.cpuview_regs_base == Radix::Bin { 500.0 } else { 120.0 })
                .show(ctx, |ui| {
                    //self.stateview(ui);
                    self.cpuview.ui(ui, &mut self.config, &self.tx_ctrl);
                });

            // IO Panel
            egui::SidePanel::right("io_panel")
                .resizable(false)
                .max_width(128.0)
                .show(ctx, |ui| {
                    self.dev_legacyio.gui_panel(ctx, ui);
                    ui.separator();
                });

            // Main Panel
            egui::CentralPanel::default().show(ctx, |ui| {
                self.graphicsview.ui(ui, &mut self.config, &self.tx_ctrl);
                self.memoryview.ui(ui, &mut self.config, &self.tx_ctrl);
            });
        });
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
