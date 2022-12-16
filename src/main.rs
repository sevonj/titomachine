/*
 * The rust GUI side of is a mess.
 * Once full fuctionality has been achieved:
 * TODO: move titolib bindings to a separate file.
 * TODO: make the gui code suck less.
 */

use egui::Stroke;
use egui_extras::{Column, TableBuilder};
pub mod titolib;
use titolib::*;

#[derive(PartialEq, Default)]
enum base {
    bin,
    #[default]
    dec,
    hex,
}

#[derive(Default)]
struct TitoApp {
    pub filepath_input: String,
    pub filepath_current: String,
    pub autoplay: bool,
    pub autoplay_speed: f32,
    pub autoplay_str: String,
    pub kbdin: String,
    pub crtout: String,

    pub memview_adr_base: base,
    pub memview_val_base: base,
    pub register_val_base: base,
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Main Window space
        egui::CentralPanel::default().show(ctx, |ui| {
            // Toolbar
            egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.menu_button("File", |ui| {
                        ui.menu_button("Open", |ui| {
                            ui.allocate_space(egui::vec2(256.0, 0.0));
                            ui.label(
                                "No proper file dialog yet. Instead, paste your filepath here.",
                            );
                            egui::TextEdit::singleline(&mut self.filepath_input)
                                .hint_text("filepath.")
                                .show(ui);
                            if ui.button("Load this file").clicked() {
                                self.filepath_current = self.filepath_input.as_str().to_owned();
                                self.filepath_input = String::new();
                                tito_loadprog(self.filepath_current.as_str());
                                ui.close_menu();
                            }
                        });
                        if ui.add(egui::Button::new("Clear")).clicked() {
                            tito_clearmem();
                        }
                    });
                    ui.menu_button("Options", |ui| {
                        ui.menu_button("Memview", |ui| {
                            ui.label("Memview Address base");
                            ui.radio_value(&mut self.memview_adr_base, base::bin, "Binary");
                            ui.radio_value(&mut self.memview_adr_base, base::dec, "Decimal");
                            ui.radio_value(&mut self.memview_adr_base, base::hex, "Hex");
                            ui.label("Memview Value base");
                            ui.radio_value(&mut self.memview_val_base, base::bin, "Binary");
                            ui.radio_value(&mut self.memview_val_base, base::dec, "Decimal");
                            ui.radio_value(&mut self.memview_val_base, base::hex, "Hex");
                            ui.label("Register Value base");
                            ui.radio_value(&mut self.register_val_base, base::bin, "Binary");
                            ui.radio_value(&mut self.register_val_base, base::dec, "Decimal");
                            ui.radio_value(&mut self.register_val_base, base::hex, "Hex");
                        });
                        ui.menu_button("Language", |ui| {
                            ui.label("Nou Finland Längvits :(");
                            ui.add_enabled_ui(false, |ui| {
                                ui.radio(true, "EN (English)");
                                ui.radio(false, "FI (Suomi)");
                            });
                        });
                    });
                    //ui.button("Compile");
                    if tito_readcureg(0) < 0 {
                        if ui.add(egui::Button::new("Start")).clicked() {
                            tito_start();
                        }
                    } else {
                        if ui.add(egui::Button::new("Stop")).clicked() {
                            tito_stop();
                        }
                    }
                    if self.autoplay {
                        if ui.add(egui::Button::new("Play")).clicked() {
                            tito_tick();
                        }
                    } else {
                        if ui.add(egui::Button::new("Tick")).clicked() {
                            tito_tick();
                        }
                    }
                    ui.add(egui::Checkbox::new(&mut self.autoplay, "Autoplay"));
                    if self.autoplay {
                        ui.label("Speed limit: ");
                        egui::TextEdit::singleline(&mut self.autoplay_str)
                            .desired_width(32.0)
                            .show(ui);
                        self.autoplay_speed = self
                            .autoplay_str
                            .parse::<f32>()
                            .unwrap_or(self.autoplay_speed);
                        ui.label("(instructions / sec)");
                    }

                    ui.menu_button("Help", |ui| {
                        if ui.button("TTK-91 Instructions").clicked() {
                            open::that(
                                "https://www.cs.helsinki.fi/group/titokone/ttk91_ref_fi.html",
                            );
                        }
                        ui.label("Nothing below works yet.");

                        ui.button("View Hotkeys");
                        ui.button("About");
                    });
                });
            });
            // Main Panel
            egui::CentralPanel::default().show(ctx, |ui| {
                // Register Panel
                egui::SidePanel::right("register_panel")
                    .resizable(false)
                    .show(ctx, |ui| {
                        // CPU Registers
                        ui.label("CPU Registers");

                        TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::initial(48.0))
                            .column(Column::initial(48.0))
                            .body(|mut body| {
                                for i in (0..8) {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(format!("R{:x}", i));
                                        });
                                        row.col(|ui| {
                                            ui.label(tito_readreg(i).to_string());
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
                    if self.memview_val_base == base::bin {
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
                            for i in (0..rowcount) {
                                body.row(20.0, |mut row| {
                                    let val: i32 = tito_readmem(i);
                                    let mut regstr = String::new();
                                    let point_pc: bool = (tito_readcureg(0) == i);
                                    let point_sp: bool = (tito_readreg(6) == i);
                                    let point_fp: bool = (tito_readreg(7) == i);
                                    if (point_pc || point_sp || point_fp) {
                                        regstr.push_str("<-- ")
                                    }
                                    if point_pc {
                                        regstr.push_str("PC ")
                                    }
                                    if point_sp {
                                        regstr.push_str("SP ")
                                    }
                                    if point_sp {
                                        regstr.push_str("FP ")
                                    }
                                    let mut mem_adr_str = String::new();
                                    if self.memview_adr_base == base::bin {
                                        mem_adr_str.push_str(format!("{i:#b}").as_str());
                                    } else if self.memview_adr_base == base::dec {
                                        mem_adr_str.push_str(format!("{i}").as_str());
                                    } else if self.memview_adr_base == base::hex {
                                        mem_adr_str.push_str(format!("{i:#x}").as_str());
                                    }
                                    let mut mem_val_str = String::new();
                                    if self.memview_val_base == base::bin {
                                        mem_val_str.push_str(format!("{val:#034b}").as_str());
                                    } else if self.memview_val_base == base::dec {
                                        mem_val_str.push_str(format!("{val}").as_str());
                                    } else if self.memview_val_base == base::hex {
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
        "TitoGui",
        native_options,
        Box::new(|cc| Box::new(TitoApp::new(cc))),
    );
}
