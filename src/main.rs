/*
 * The rust GUI side of is a mess.
 * Once full fuctionality has been achieved:
 * TODO: move titolib bindings to a separate file.
 * TODO: make the gui code suck less.
 */

use egui_extras::{Column, TableBuilder};
pub mod titolib;
use titolib::*;

use egui::Color32;
use egui::FontId;

use std::env;

#[derive(PartialEq, Default)]
enum Base {
    Bin,
    #[default]
    Dec,
    Hex,
}

pub const SHORTCUT_OPEN: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::O);
pub const SHORTCUT_CLEAR: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::E);

pub const SHORTCUT_COMPILE: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::K);

pub const SHORTCUT_START: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::R);
pub const SHORTCUT_STOP: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape);
pub const SHORTCUT_TICK: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Space);
pub const SHORTCUT_AUTOPLAY: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::R);

//#[derive(Default)]
struct TitoApp {
    pub filepath_current: String,

    pub autoplay: bool,
    pub autoplay_speed: i32,
    pub autoplay_playing: bool,
    pub kbdin: String,

    pub memview_adr_base: Base,
    pub memview_val_base: Base,
    pub register_val_base: Base,
}
impl Default for TitoApp {
    fn default() -> Self {
        TitoApp {
            filepath_current: env::current_dir().unwrap().to_str().unwrap().to_owned(),
            autoplay: false,
            autoplay_speed: 1,
            autoplay_playing: false,
            kbdin: String::new(),
            memview_adr_base: Base::Dec,
            memview_val_base: Base::Dec,
            register_val_base: Base::Dec,
        }
    }
}
impl TitoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        //cc.egui_ctx.set_fonts(egui::FontDefinitions { font_data: (), families: () });
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        Self::default()
    }
}

impl eframe::App for TitoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let tito_running: bool = tito_readcureg(0) >= 0;

            // Shortcuts
            if ui.input_mut().consume_shortcut(&SHORTCUT_OPEN) {
                match tinyfiledialogs::open_file_dialog(
                    "Open",
                    self.filepath_current.as_str(),
                    None,
                ) {
                    Some(file) => {
                        tito_loadprog(&file);
                        self.filepath_current = file.to_owned();
                    }
                    None => (),
                }
            }
            if ui.input_mut().consume_shortcut(&SHORTCUT_CLEAR) {
                tito_clearmem()
            }
            if ui.input_mut().consume_shortcut(&SHORTCUT_STOP) {
                tito_stop();
            }
            if ui.input_mut().consume_shortcut(&SHORTCUT_START) {
                if tito_running {
                    tito_stop()
                } else {
                    tito_start();
                }
            }
            if ui.input_mut().consume_shortcut(&SHORTCUT_AUTOPLAY) {
                self.autoplay = !self.autoplay;
            }

            if self.autoplay {
                if ui.input_mut().consume_shortcut(&SHORTCUT_TICK) {
                    self.autoplay_playing = !self.autoplay_playing;
                }
            } else {
                if ui.input_mut().consume_shortcut(&SHORTCUT_TICK) {
                    tito_tick()
                }
            }

            // Toolbar
            egui::TopBottomPanel::top("toolbar")
                .exact_height(32.0)
                .show(ctx, |ui| {
                    //                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.menu_button("File", |ui| {
                            if ui
                                .add(
                                    egui::Button::new("Open File")
                                        .shortcut_text(ctx.format_shortcut(&SHORTCUT_OPEN)),
                                )
                                .clicked()
                            {
                                match tinyfiledialogs::open_file_dialog(
                                    "Open",
                                    self.filepath_current.as_str(),
                                    None,
                                ) {
                                    Some(file) => {
                                        tito_loadprog(&file);
                                        self.filepath_current = file.to_owned();
                                    }
                                    None => (),
                                }
                                ui.close_menu();
                            }
                            if ui
                                .add(
                                    egui::Button::new("Clear Memory")
                                        .shortcut_text(ctx.format_shortcut(&SHORTCUT_CLEAR)),
                                )
                                .clicked()
                            {
                                tito_clearmem();
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
                            ui.menu_button("Language", |ui| {
                                ui.add_enabled_ui(false, |ui| {
                                    ui.label("no language support")
                                    //ui.radio(true, "EN (English)");
                                    //ui.radio(false, "FI (Suomi)");
                                });
                            });
                        });
                        ui.menu_button("Help", |ui| {
                            if ui.button("↗TTK-91 Instructions").clicked() {
                                ui.output().open_url(
                                    "https://www.cs.helsinki.fi/group/titokone/ttk91_ref_fi.html",
                                );
                            }
                        });

                        ui.separator();

                        //ui.button("Compile");
                        if tito_running {
                            if ui
                                .add(egui::Button::new(
                                    egui::RichText::new("⏼on/off").color(Color32::WHITE),
                                ))
                                .clicked()
                            {
                                self.autoplay_playing = false;
                                if tito_running {
                                    tito_stop();
                                } else {
                                    tito_start();
                                }
                            }
                        } else {
                            if ui.add(egui::Button::new("⏼on/off")).clicked() {
                                self.autoplay_playing = false;
                                if tito_running {
                                    tito_stop();
                                } else {
                                    tito_start();
                                }
                            }
                        }
                        ui.add_enabled_ui(tito_running, |ui| {
                            if self.autoplay {
                                if self.autoplay_playing {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("⏸")
                                                    .color(Color32::WHITE)
                                                    .font(FontId::monospace(16.0)),
                                            )
                                            .min_size(egui::vec2(24.0, 0.0)),
                                        )
                                        .clicked()
                                    {
                                        self.autoplay_playing = false;
                                    }
                                } else {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("⏵")
                                                    .font(FontId::monospace(16.0)),
                                            )
                                            .min_size(egui::vec2(24.0, 0.0)),
                                        )
                                        .clicked()
                                    {
                                        self.autoplay_playing = true;
                                    }
                                }
                            } else {
                                self.autoplay_playing = false;
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("‣").font(FontId::monospace(16.0)),
                                        )
                                        .min_size(egui::vec2(24.0, 0.0)),
                                    )
                                    .clicked()
                                {
                                    tito_tick();
                                }
                            }
                            //ui.add(egui::Checkbox::new(&mut self.autoplay, "🔁"));
                            if self.autoplay {
                                if ui
                                    .add(egui::Button::new(
                                        egui::RichText::new("🔁")
                                            .color(Color32::LIGHT_BLUE)
                                            .font(FontId::monospace(16.0)),
                                    ))
                                    .clicked()
                                {
                                    self.autoplay = false;
                                }
                            } else if ui
                                .add(egui::Button::new(
                                    egui::RichText::new("🔁").font(FontId::monospace(16.0)),
                                ))
                                .clicked()
                            {
                                self.autoplay = true;
                            }
                        });
                        ui.allocate_space(egui::vec2(ui.available_width() - 128.0, 0.0));
                        ui.add_enabled_ui(self.autoplay, |ui| {
                            ui.label("Speed limit: ");
                            ui.add(
                                egui::DragValue::new(&mut self.autoplay_speed)
                                    .speed(0.1)
                                    .clamp_range(1..=999),
                            );
                            //ui.label("(instructions / sec)");
                        });
                    });
                });
            // Main Panel
            egui::CentralPanel::default().show(ctx, |_ui| {
                // Register Panel
                egui::SidePanel::right("register_panel")
                    .resizable(false)
                    .show(ctx, |ui| {
                        // CPU Registers
                        ui.label("CPU Registers");
                        let reg_col_width: f32;
                        if self.register_val_base == Base::Bin {
                            reg_col_width = 240.0;
                        } else {
                            reg_col_width = 72.0;
                        }
                        TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::initial(16.0))
                            .column(Column::exact(reg_col_width))
                            .body(|mut body| {
                                for i in 0..8 {
                                    let val = tito_readreg(i);
                                    let mut reg_val_str = String::new();
                                    if self.register_val_base == Base::Bin {
                                        reg_val_str.push_str(format!("{val:#034b}").as_str());
                                    } else if self.register_val_base == Base::Dec {
                                        reg_val_str.push_str(format!("{val}").as_str());
                                    } else if self.register_val_base == Base::Hex {
                                        reg_val_str.push_str(format!("{val:#010x}").as_str());
                                    }
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(format!("R{i}"));
                                        });
                                        row.col(|ui| {
                                            ui.label(reg_val_str);
                                        });
                                    });
                                }
                            });
                    });
                // IO Panel
                egui::SidePanel::right("io_panel")
                    .resizable(false)
                    .max_width(128.0)
                    .show(ctx, |ui| {
                        ui.label("=CRT");
                        // =CRT
                        egui::Frame::side_top_panel(&ctx.style())
                            .fill(egui::Color32::BLACK)
                            .show(ui, |ui| {
                                ui.label(tito_output_buffer());
                                ui.allocate_space(egui::vec2(ui.available_width(), 0.0))
                            });

                        ui.separator();
                        //let mut kbd_input = "";
                        let kbdenable = tito_is_waiting_for_input();
                        if kbdenable {
                            ui.label(
                                egui::RichText::new("=KBD")
                                    .font(egui::FontId::monospace(12.0))
                                    .color(egui::Color32::WHITE),
                            );
                        } else {
                            ui.label("=KBD");
                        }
                        ui.add_enabled_ui(kbdenable, |ui| {
                            egui::TextEdit::singleline(&mut self.kbdin)
                                .hint_text("Type a number")
                                .show(ui);
                            if ui.button("Send").clicked() {
                                if self.kbdin.parse::<i32>().is_ok() {
                                    tito_input(self.kbdin.parse::<i32>().unwrap());
                                    self.kbdin = String::new();
                                } else {
                                    self.kbdin = "Invalid input!".to_owned();
                                }
                            }
                        });
                        ui.separator();
                    });
                // Memory View Panel
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Memory View Table
                    let memview_val_width: f32;
                    if self.memview_val_base == Base::Bin {
                        memview_val_width = 256.0;
                    } else {
                        memview_val_width = 96.0
                    }
                    TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::exact(96.0)) // Address
                        .column(Column::exact(memview_val_width)) // Value
                        .column(Column::exact(192.0)) // Instruction
                        .column(Column::remainder()) // Registers PC/SP/FP
                        .header(20.0, |mut header| {
                            // If PC is at this address, draw highlighted row.
                            header.col(|ui| {
                                ui.heading(
                                    egui::RichText::new("Address")
                                        .font(egui::FontId::proportional(12.5)),
                                );
                            });
                            header.col(|ui| {
                                ui.heading(
                                    egui::RichText::new("Value")
                                        .font(egui::FontId::proportional(12.5)),
                                );
                            });
                            header.col(|ui| {
                                ui.heading(
                                    egui::RichText::new("Decoded Instruction")
                                        .font(egui::FontId::proportional(12.5)),
                                );
                            });
                            header.col(|ui| {
                                ui.heading(
                                    egui::RichText::new("").font(egui::FontId::proportional(12.5)),
                                );
                            });
                        })
                        .body(|mut body| {
                            let rowcount = 128;
                            for i in 0..rowcount {
                                body.row(20.0, |mut row| {
                                    let val: i32 = tito_readmem(i);
                                    let mut regstr = String::new();
                                    let point_pc: bool = tito_readcureg(0) == i;
                                    let point_sp: bool = tito_readreg(6) == i;
                                    let point_fp: bool = tito_readreg(7) == i;
                                    if point_pc || point_sp || point_fp {
                                        regstr.push_str("<-- ")
                                    }
                                    if point_pc {
                                        regstr.push_str("PC ")
                                    }
                                    if point_sp {
                                        regstr.push_str("SP ")
                                    }
                                    if point_fp {
                                        regstr.push_str("FP ")
                                    }
                                    let mut mem_adr_str = String::new();
                                    if self.memview_adr_base == Base::Bin {
                                        mem_adr_str.push_str(format!("{i:#b}").as_str());
                                    } else if self.memview_adr_base == Base::Dec {
                                        mem_adr_str.push_str(format!("{i}").as_str());
                                    } else if self.memview_adr_base == Base::Hex {
                                        mem_adr_str.push_str(format!("{i:#x}").as_str());
                                    }
                                    let mut mem_val_str = String::new();
                                    if self.memview_val_base == Base::Bin {
                                        mem_val_str.push_str(format!("{val:#034b}").as_str());
                                    } else if self.memview_val_base == Base::Dec {
                                        mem_val_str.push_str(format!("{val}").as_str());
                                    } else if self.memview_val_base == Base::Hex {
                                        mem_val_str.push_str(format!("{val:#010x}").as_str());
                                    }

                                    if i == tito_readcureg(0) {
                                        row.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(mem_adr_str)
                                                    .font(egui::FontId::monospace(12.0))
                                                    .color(egui::Color32::WHITE),
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(mem_val_str)
                                                    .font(egui::FontId::monospace(12.0))
                                                    .color(egui::Color32::WHITE),
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(tito_inst_to_string(val))
                                                    .font(egui::FontId::monospace(12.0))
                                                    .color(egui::Color32::WHITE),
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(regstr)
                                                    .font(egui::FontId::monospace(12.0))
                                                    .color(egui::Color32::WHITE),
                                            );
                                        });
                                    } else {
                                        row.col(|ui| {
                                            //ui.label(format!("0x{:01$x}", i, 4));
                                            ui.label(
                                                egui::RichText::new(mem_adr_str)
                                                    .font(egui::FontId::monospace(12.0)),
                                            );
                                            ui.allocate_space(ui.available_size());
                                        });
                                        row.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(mem_val_str)
                                                    .font(egui::FontId::monospace(12.0)),
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(tito_inst_to_string(val))
                                                    .font(egui::FontId::monospace(12.0)),
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(regstr)
                                                    .font(egui::FontId::monospace(12.0)),
                                            );
                                        });
                                    }
                                });
                            }
                        });
                });
            });
        });
    }
}

fn main() {
    tito_clearmem();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "TiToMachine",
        native_options,
        Box::new(|cc| Box::new(TitoApp::new(cc))),
    );
}
