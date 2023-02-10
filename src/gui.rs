use super::emulator::CtrlMSG;
use crate::{emulator::ReplyMSG, TitoApp};
use serde;
use std::{
    env::{self, current_dir},
    path::PathBuf,
    time::Duration,
};
pub mod gui_editor;
pub mod gui_emulator;

use egui::{Button, Color32, Modifiers};
use rfd;

#[derive(PartialEq)]
pub enum GuiMode {
    Editor,
    Emulator,
}
#[derive(PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum Base {
    Bin,
    #[default]
    Dec,
    Hex,
}

pub const SHORTCUT_OPEN: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::O);
pub const SHORTCUT_SAVE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::S);
pub const SHORTCUT_SAVEAS: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), egui::Key::S);
pub const SHORTCUT_CLEAR: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::E);
pub const SHORTCUT_COMPILE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::K);
pub const SHORTCUT_START: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::R);
pub const SHORTCUT_STOP: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Escape);
pub const SHORTCUT_TICK: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Enter);
pub const SHORTCUT_PLAY: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Space);

impl TitoApp {
    pub fn gui_main(&mut self, ctx: &egui::Context) {
        // 60fps gui update when emulator is running
        if self.emu_running && self.emu_playing {
            ctx.request_repaint_after(Duration::from_secs(1 / 60))
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            // Toolbar
            egui::TopBottomPanel::top("toolbar")
                .exact_height(32.0)
                .show(ctx, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        // File, Options, Help
                        self.gui_menubar_entries(ctx, ui);
                        ui.separator();
                        // Run, Edit
                        ui.selectable_value(&mut self.guimode, GuiMode::Editor, "Edit");
                        ui.selectable_value(&mut self.guimode, GuiMode::Emulator, "Run");
                        ui.separator();
                        // Context toolbar
                        if self.guimode == GuiMode::Emulator {
                            self.emulator_toolbar(ctx, ui);
                        } else {
                            self.editor_toolbar(ctx, ui);
                        }
                    });
                });
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.guimode == GuiMode::Emulator {
                    self.emulator_panel(ctx, ui);
                } else {
                    if self.emu_running {
                        self.emu_tx.send(CtrlMSG::Stop);
                        self.emu_running = false;
                    }
                    self.editor_panel(ctx, ui);
                }
            });
        });
    }
    fn gui_menubar_entries(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui
                .add(Button::new("Open File").shortcut_text(ctx.format_shortcut(&SHORTCUT_OPEN)))
                .clicked()
            {
                self.editor.open_file(
                    rfd::FileDialog::new()
                        .add_filter("TTK Source files", &["k91"])
                        .set_directory(&self.working_dir)
                        .pick_file(),
                );
                self.guimode = GuiMode::Editor;
                self.working_dir = current_dir().unwrap();
                ui.close_menu();
            }
            if ui
                .add_enabled(
                    self.editor.source_path != None,
                    Button::new("Save").shortcut_text(ctx.format_shortcut(&SHORTCUT_SAVE)),
                )
                .clicked()
            {
                self.editor.save_file(None);
                ui.close_menu();
            }
            if ui
                .add(Button::new("Save As").shortcut_text(ctx.format_shortcut(&SHORTCUT_SAVEAS)))
                .clicked()
            {
                self.editor.save_file(
                    rfd::FileDialog::new()
                        .add_filter("TTK Source files", &["k91"])
                        .set_directory(&self.working_dir)
                        .save_file(),
                );
                self.working_dir = current_dir().unwrap();
                ui.close_menu();
            }
        });
        ui.menu_button("Options", |ui| {
            ui.menu_button("Memory View", |ui| {
                ui.label("Memview Address base");
                ui.radio_value(&mut self.memview_adr_base, Base::Bin, "Binary");
                ui.radio_value(&mut self.memview_adr_base, Base::Dec, "Decimal");
                ui.radio_value(&mut self.memview_adr_base, Base::Hex, "Hex");
                ui.label("Memview Value base");
                ui.radio_value(&mut self.memview_val_base, Base::Bin, "Binary");
                ui.radio_value(&mut self.memview_val_base, Base::Dec, "Decimal");
                ui.radio_value(&mut self.memview_val_base, Base::Hex, "Hex");
                ui.label("Register Value base");
                ui.radio_value(&mut self.register_val_base, Base::Bin, "Binary");
                ui.radio_value(&mut self.register_val_base, Base::Dec, "Decimal");
                ui.radio_value(&mut self.register_val_base, Base::Hex, "Hex");
            });

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("CPU Speed: ");
                if ui
                    .add_enabled(
                        !self.emu_turbo,
                        egui::DragValue::new(&mut self.emu_play_speed)
                            .speed(0.1)
                            .clamp_range(1..=9999),
                    )
                    .changed()
                {
                    if self.emu_use_khz {
                        self.emu_tx
                            .send(CtrlMSG::SetRate(self.emu_play_speed * 1000.));
                    } else {
                        self.emu_tx.send(CtrlMSG::SetRate(self.emu_play_speed));
                    }
                }
                if self.emu_use_khz {
                    ui.label("KHz");
                } else {
                    ui.label("Hz");
                }
            });
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.radio_value(&mut self.emu_use_khz, false, "Hz");
                if ui.radio_value(&mut self.emu_use_khz, true, "KHz").changed() {
                    if self.emu_use_khz {
                        self.emu_tx
                            .send(CtrlMSG::SetRate(self.emu_play_speed * 1000.));
                    } else {
                        self.emu_tx.send(CtrlMSG::SetRate(self.emu_play_speed));
                    }
                };
            });
            if ui.checkbox(&mut self.emu_turbo, "Turbo Mode").changed() {
                self.emu_tx.send(CtrlMSG::SetTurbo(self.emu_turbo));
            };
            ui.menu_button("Language", |ui| {
                ui.add_enabled_ui(false, |ui| {
                    ui.label("no language support")
                    //ui.radio(true, "EN (English)");
                    //ui.radio(false, "FI (Suomi)");
                });
            });
        });
        ui.menu_button("Help", |ui| {
            if ui.button("↗TTK-91 Reference").clicked() {
                ui.output()
                    .open_url("https://www.cs.helsinki.fi/group/titokone/ttk91_ref_fi.html");
            }
        });
    }
}
