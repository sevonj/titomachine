use super::emulator::CtrlMSG;
use crate::TitoApp;
use serde;
pub mod file_actions;
pub mod gui_editor;
pub mod gui_emulator;

use egui::{Align, Button, DragValue, Layout, Modifiers};
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

pub const SHORTCUT_NEW: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::N);
pub const SHORTCUT_OPEN: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::O);
pub const SHORTCUT_SAVE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::S);
pub const SHORTCUT_SAVEAS: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), egui::Key::S);

pub const SHORTCUT_GUI_EDIT: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::E);
pub const SHORTCUT_GUI_RUN: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::R);
pub const SHORTCUT_GUI_EMUGRAPHICS: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::G);
//pub const SHORTCUT_CLEAR: egui::KeyboardShortcut =
//    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::E);
pub const SHORTCUT_COMPILE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::B);

//pub const SHORTCUT_START: egui::KeyboardShortcut =
//    egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Escape);
pub const SHORTCUT_STOP: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Escape);
pub const SHORTCUT_TOGGLEPOWER: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::T);
pub const SHORTCUT_PLAY: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Space);
pub const SHORTCUT_TICK: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Enter);

impl TitoApp {
    pub fn gui_main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.consume_shortcuts(ctx, ui);

            // Toolbar
            egui::TopBottomPanel::top("toolbar")
                .exact_height(32.0)
                .show(ctx, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        // File, Options, Help
                        self.gui_menubar_entries(ctx, ui);
                        ui.separator();
                        // Edit <-> Run
                        ui.selectable_value(&mut self.guimode, GuiMode::Editor, "Edit");
                        ui.selectable_value(&mut self.guimode, GuiMode::Emulator, "Run");
                        ui.separator();
                        // Context toolbar
                        match self.guimode == GuiMode::Emulator {
                            true => self.emulator_toolbar(ctx, ui),
                            false => self.editor_toolbar(ctx, ui),
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
                .add(Button::new("New").shortcut_text(ctx.format_shortcut(&SHORTCUT_NEW)))
                .clicked()
            {
                self.file_new();
                ui.close_menu();
            }
            if ui
                .add(Button::new("Open").shortcut_text(ctx.format_shortcut(&SHORTCUT_OPEN)))
                .clicked()
            {
                self.file_open();
                ui.close_menu();
            }
            if ui
                .add_enabled(
                    self.editor.source_path != None,
                    Button::new("Save").shortcut_text(ctx.format_shortcut(&SHORTCUT_SAVE)),
                )
                .clicked()
            {
                self.file_save();
                ui.close_menu();
            }
            if ui
                .add(Button::new("Save As").shortcut_text(ctx.format_shortcut(&SHORTCUT_SAVEAS)))
                .clicked()
            {
                self.file_saveas();
                ui.close_menu();
            }
        });

        ui.menu_button("Options", |ui| {
            ui.menu_button("Memory View", |ui| {
                ui.label("Memview Address base");
                ui.radio_value(&mut self.mem_adr_base, Base::Bin, "Binary");
                ui.radio_value(&mut self.mem_adr_base, Base::Dec, "Decimal");
                ui.radio_value(&mut self.mem_adr_base, Base::Hex, "Hex");
                ui.label("Memview Value base");
                ui.radio_value(&mut self.mem_val_base, Base::Bin, "Binary");
                ui.radio_value(&mut self.mem_val_base, Base::Dec, "Decimal");
                ui.radio_value(&mut self.mem_val_base, Base::Hex, "Hex");
                ui.label("Register Value base");
                ui.radio_value(&mut self.regs_base, Base::Bin, "Binary");
                ui.radio_value(&mut self.regs_base, Base::Dec, "Decimal");
                ui.radio_value(&mut self.regs_base, Base::Hex, "Hex");
            });

            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                ui.label("CPU Speed: ");
                if ui
                    .add_enabled(
                        !self.emu_turbo,
                        DragValue::new(&mut self.emu_speed)
                            .speed(0.1)
                            .clamp_range(1..=9999),
                    )
                    .changed()
                {
                    self.send_settings()
                }
                match self.emu_use_khz {
                    true => ui.label("KHz"),
                    false => ui.label("Hz"),
                }
            });
            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                ui.radio_value(&mut self.emu_use_khz, false, "Hz");
                if ui.radio_value(&mut self.emu_use_khz, true, "KHz").changed() {
                    if self.emu_use_khz {
                        self.emu_tx.send(CtrlMSG::SetRate(self.emu_speed * 1000.));
                    } else {
                        self.emu_tx.send(CtrlMSG::SetRate(self.emu_speed));
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

    fn consume_shortcuts(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // General
        if ui.input_mut().consume_shortcut(&SHORTCUT_NEW) {
            self.file_new()
        }
        if ui.input_mut().consume_shortcut(&SHORTCUT_OPEN) {
            self.file_open()
        }
        if ui.input_mut().consume_shortcut(&SHORTCUT_SAVE) && self.editor.source_path != None {
            self.file_save()
        }
        if ui.input_mut().consume_shortcut(&SHORTCUT_SAVEAS) {
            self.file_saveas()
        }
        if ui.input_mut().consume_shortcut(&SHORTCUT_GUI_EDIT) {
            self.guimode = GuiMode::Editor
        }
        if ui.input_mut().consume_shortcut(&SHORTCUT_GUI_RUN) {
            self.guimode = GuiMode::Emulator
        }
        // Editor specific
        if self.guimode == GuiMode::Editor {
            if ui.input_mut().consume_shortcut(&SHORTCUT_COMPILE) {
                self.file_compile()
            }
        }
        // Emulator specific
        else {
            if ui.input_mut().consume_shortcut(&SHORTCUT_GUI_EMUGRAPHICS) {
                self.emugui_display = !self.emugui_display
            }
            if ui.input_mut().consume_shortcut(&SHORTCUT_TOGGLEPOWER) {
                match self.emu_running {
                    true => {
                        self.emu_tx.send(CtrlMSG::Stop);
                    }
                    false => {
                        self.emu_tx.send(CtrlMSG::Start);
                    }
                }
            }
            if ui.input_mut().consume_shortcut(&SHORTCUT_STOP) {
                self.emu_tx.send(CtrlMSG::Stop);
            }
            if self.emu_running {
                if ui.input_mut().consume_shortcut(&SHORTCUT_PLAY) {
                    self.emu_tx.send(CtrlMSG::PlayPause(!self.emu_playing));
                }
                if ui.input_mut().consume_shortcut(&SHORTCUT_TICK) && !self.emu_playing {
                    self.emu_tx.send(CtrlMSG::Tick);
                    ctx.request_repaint_after(std::time::Duration::from_secs(1 / 60))
                }
            }
        }
    }
}
