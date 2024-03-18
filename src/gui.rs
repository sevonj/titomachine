use crate::{emulator::emu_debug::CtrlMSG, TitoApp};
use serde;

pub mod gui_editor;
pub mod gui_emulator;

use egui::{Align, Button, Color32, DragValue, Layout, Modifiers, OpenUrl, RichText};

#[derive(PartialEq)]
pub enum GuiMode {
    Editor,
    Emulator,
}

#[derive(PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum Radix {
    Bin,
    #[default]
    Dec,
    Hex,
}

impl Radix {
    pub fn format_i32(&self, value: i32) -> String {
        match self {
            Radix::Bin => format!("{value:#034b}"),
            Radix::Dec => format!("{value}"),
            Radix::Hex => format!("{value:#010x}"),
        }
    }

    /// Same as above, but expects usize and only adds 16 bits worth of leading zeros.
    pub fn format_addr(&self, value: usize) -> String {
        match self {
            Radix::Bin => format!("{value:#b}"),
            Radix::Dec => format!("{value}"),
            Radix::Hex => format!("{value:#x}"),
        }
    }
}

const URL_GITHUB: &str = "https://github.com/sevonj/titomachine/";
const URL_GUIDE: &str = "https://sevonj.github.io/titouserdoc/";
const URL_OLDREF: &str = "https://www.cs.helsinki.fi/group/titokone/ttk91_ref_fi.html";

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

pub const SHORTCUT_DEBUG_GUI: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::ALT), egui::Key::D);

impl TitoApp {
    pub fn gui_main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.consume_shortcuts(ctx, ui);

            // Toolbar
            egui::TopBottomPanel::top("toolbar")
                .exact_height(32.0)
                .show(ctx, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!("assets/32bit.png"))
                                .fit_to_original_size(0.75)
                                .tint(Color32::DARK_GRAY)
                        );
                        ui.separator();
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
            // Bottom bar
            egui::TopBottomPanel::bottom("bottombar")
                .exact_height(24.0)
                .show(ctx, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        // Filename
                        ui.label(&self.filestatus.displayname);
                        // Compile Status
                        match self.guimode {
                            GuiMode::Editor => {
                                if self.filestatus.compilefail {
                                    ui.label(
                                        RichText::new("Could not compile!").color(Color32::RED),
                                    );
                                }
                            }
                            GuiMode::Emulator => {
                                if self.filestatus.uncompiled {
                                    ui.label(
                                        RichText::new("File has uncompiled changes!")
                                            .color(Color32::YELLOW),
                                    );
                                }
                            }
                        }
                    });
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                if self.guimode == GuiMode::Emulator {
                    self.emulator_panel(ctx, ui);
                } else {
                    if self.emu_running {
                        let _ = self.tx_ctrl.send(CtrlMSG::PlaybackStop);
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
            ui.label("Editor");
            ui.menu_button("Compiler", |ui| {
                ui.checkbox(
                    &mut self.editorsettings.compile_default_os,
                    "Use default SVCs",
                );
            });
            ui.separator();
            ui.label("Emulator");
            ui.menu_button("Memory View", |ui| {
                ui.checkbox(&mut self.emugui_follow_pc, "Follow PC");
                ui.label("Memview Address base");
                ui.radio_value(&mut self.mem_adr_base, Radix::Bin, "Binary");
                ui.radio_value(&mut self.mem_adr_base, Radix::Dec, "Decimal");
                ui.radio_value(&mut self.mem_adr_base, Radix::Hex, "Hex");
                ui.label("Memview Value base");
                ui.radio_value(&mut self.mem_val_base, Radix::Bin, "Binary");
                ui.radio_value(&mut self.mem_val_base, Radix::Dec, "Decimal");
                ui.radio_value(&mut self.mem_val_base, Radix::Hex, "Hex");
                ui.label("Register Value base");
                ui.radio_value(&mut self.regs_base, Radix::Bin, "Binary");
                ui.radio_value(&mut self.regs_base, Radix::Dec, "Decimal");
                ui.radio_value(&mut self.regs_base, Radix::Hex, "Hex");
            });

            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                ui.label("CPU Speed: ");
                ui.add_enabled(
                    !self.emu_turbo,
                    DragValue::new(&mut self.emu_speed)
                        .speed(0.1)
                        .clamp_range(1..=9999),
                );
                match self.emu_cpuspeedmul {
                    crate::FreqMagnitude::Hz => ui.label("Hz"),
                    crate::FreqMagnitude::KHz => ui.label("KHz"),
                    crate::FreqMagnitude::MHz => ui.label("MHz"),
                }
            });
            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                if ui
                    .radio_value(&mut self.emu_cpuspeedmul, crate::FreqMagnitude::Hz, "Hz")
                    .clicked()
                {
                    self.send_settings();
                }
                if ui
                    .radio_value(&mut self.emu_cpuspeedmul, crate::FreqMagnitude::KHz, "KHz")
                    .clicked()
                {
                    self.send_settings();
                }
                if ui
                    .radio_value(&mut self.emu_cpuspeedmul, crate::FreqMagnitude::MHz, "MHz")
                    .clicked()
                {
                    self.send_settings();
                }
            });
            if ui.checkbox(&mut self.emu_turbo, "Turbo Mode").changed() {
                let _ = self.tx_ctrl.send(CtrlMSG::SetTurbo(self.emu_turbo));
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
            if ui.button("↗User Guide").clicked() {
                ui.output_mut(|o| o.open_url = Some(OpenUrl {
                    url: URL_GUIDE.into(),
                    new_tab: false,
                }));
            }
            if ui.button("↗Old TTK-91 Reference").clicked() {
                ui.output_mut(|o| o.open_url = Some(OpenUrl {
                    url: URL_OLDREF.into(),
                    new_tab: false,
                }));
            }
            if ui.button("↗Github").clicked() {
                ui.output_mut(|o| o.open_url = Some(OpenUrl {
                    url: URL_GITHUB.into(),
                    new_tab: false,
                }));
            }
        });
    }

    fn consume_shortcuts(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_DEBUG_GUI)) {
            let debug = ui.style().debug.debug_on_hover;
            ui.ctx().set_debug_on_hover(!debug);
            println!("its debuggin time, {}", debug)
        }
        // General
        if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_NEW)) {
            self.file_new()
        }
        if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_OPEN)) {
            self.file_open()
        }
        if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_SAVE)) {
            self.file_save()
        }
        if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_SAVEAS)) {
            self.file_saveas()
        }
        if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_GUI_EDIT)) {
            self.guimode = GuiMode::Editor
        }
        if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_GUI_RUN)) {
            self.guimode = GuiMode::Emulator
        }
        // Editor specific
        if self.guimode == GuiMode::Editor {
            if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_COMPILE)) {
                self.file_compile()
            }
        }
        // Emulator specific
        else {
            if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_GUI_EMUGRAPHICS)) {
                self.emugui_display = !self.emugui_display
            }
            if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_TOGGLEPOWER)) {
                match self.emu_running {
                    true => {
                        let _ = self.tx_ctrl.send(CtrlMSG::PlaybackStop);
                    }
                    false => {
                        let _ = self.tx_ctrl.send(CtrlMSG::PlaybackStart);
                    }
                }
            }
            if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_STOP)) {
                let _ = self.tx_ctrl.send(CtrlMSG::PlaybackStop);
            }
            if self.emu_running {
                if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_PLAY)) {
                    let _ = self.tx_ctrl
                        .send(CtrlMSG::PlaybackPlayPause(!self.emu_playing));
                }
                if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_TICK)) && !self.emu_playing {
                    let _ = self.tx_ctrl.send(CtrlMSG::PlaybackTick);
                    ctx.request_repaint_after(std::time::Duration::from_secs(1 / 60))
                }
            }
        }
    }
}


/// Trait for emulator GUI modules
pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}