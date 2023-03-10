/*
 * main.rs
 *
 * Project Structure:
 *
 * main.rs                  App instance, create window, start emulator thread
 *
 *   emulator.rs
 *   emulator/
 *     instance.rs          Machine instance (ram, registers, etc.)
 *     instructions.rs      Executes an instruction on machine instance.
 *     loader.rs            Load program to instance, clear instance mem, etc.
 *
 *   editor.rs
 *   editor/
 *     compiler.rs
 *
 *   gui.rs                 Main layout, common elements.
 *   gui/
 *     gui_emulator.rs      Emulator view
 *     gui_editor.rs        Editor view
 *
 *
 *
 */
#[macro_use]
extern crate num_derive;
use std::{env, path::PathBuf, sync::mpsc, thread};
pub mod editor;
pub mod emulator;
pub mod gui;
use editor::*;
use egui_extras::RetainedImage;
use emulator::{
    emu_debug::{CtrlMSG, DebugRegs, ReplyMSG},
    Emu,
};
use gui::{Base, GuiMode};
use image::{ImageBuffer, Rgba};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TitoApp {
    working_dir: PathBuf,
    // Editor
    #[serde(skip)]
    editor: Editor,
    // Emulator
    #[serde(skip)]
    emu_tx: mpsc::Sender<CtrlMSG>,
    #[serde(skip)]
    emu_rx: mpsc::Receiver<ReplyMSG>,
    #[serde(skip)]
    buf_in: String,
    #[serde(skip)]
    buf_out: String,

    current_prog: String,

    emu_running: bool,
    emu_halted: bool,
    emu_playing: bool,
    emu_use_khz: bool,
    emu_speed: f32,
    #[serde(skip)]
    emu_achieved_speed: f32,
    #[serde(skip)]
    emu_turbo: bool,
    #[serde(skip)]
    emu_regs: DebugRegs,
    #[serde(skip)]
    gui_memview: Vec<i32>, // Cached partial memory for gui
    #[serde(skip)]
    gui_memview_off: i32, // Start offset
    #[serde(skip)]
    gui_memview_len: i32, // Size of cache
    #[serde(skip)]
    emu_mem_len: usize, // Size of cache
    #[serde(skip)]
    gui_memview_scroll: f32,
    #[serde(skip)]
    emu_waiting_for_in: bool,
    #[serde(skip)]
    emu_displayimage: Option<RetainedImage>,
    #[serde(skip)]
    emu_displaybuffer: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    #[serde(skip)]
    emu_dispvec: Vec<i32>,

    //GUI
    #[serde(skip)]
    guimode: GuiMode,
    emugui_display: bool,
    emugui_follow_pc: bool,
    mem_adr_base: Base,
    mem_val_base: Base,
    regs_base: Base,
}

impl Default for TitoApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        thread::spawn(move || {
            emulator::run(tx2, rx);
        });
        TitoApp {
            working_dir: env::current_dir().unwrap(),
            // Editor
            editor: Editor::default(),
            // Emulator
            emu_tx: tx,
            emu_rx: rx2,
            buf_in: String::new(),
            buf_out: "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n".to_owned(),
            current_prog: String::new(),

            emu_running: false,
            emu_halted: false,
            emu_playing: false,
            emu_speed: 10.,
            emu_achieved_speed: 0.,
            emu_use_khz: false,
            emu_turbo: false,
            emu_regs: DebugRegs::default(),
            emu_mem_len: 0,
            gui_memview: vec![7; 16],
            gui_memview_off: 0,
            gui_memview_len: 16 as i32,
            gui_memview_scroll: 0.,
            emu_waiting_for_in: false,
            emu_displayimage: None,
            emu_displaybuffer: None,
            emu_dispvec: vec![0; 120 * 160],

            // GUI
            guimode: GuiMode::Editor,
            emugui_display: false,
            emugui_follow_pc: true,
            mem_adr_base: Base::Dec,
            mem_val_base: Base::Dec,
            regs_base: Base::Dec,
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

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn msg_handler(&mut self) {
        // Loop until there are no messages, because messages may
        // come faster than update.
        loop {
            if let Ok(msg) = self.emu_rx.try_recv() {
                match msg {
                    // Emulator State
                    ReplyMSG::State(st) => {
                        self.emu_running = st.running;
                        self.emu_halted = st.halted;
                        self.emu_playing = st.playing;
                        self.emu_achieved_speed = st.speed_percent;
                    }
                    ReplyMSG::Regs(regs) => self.emu_regs = regs,
                    ReplyMSG::Mem(vec) => self.gui_memview = vec,
                    ReplyMSG::MemSize(s) => self.emu_mem_len = s,
                    ReplyMSG::Display(vec) => {
                        self.emu_dispvec = vec;
                    }
                    // IO
                    ReplyMSG::In => self.emu_waiting_for_in = true,
                    ReplyMSG::Out(n) => {
                        self.buf_out = n.to_string() + "\n" + self.buf_out.as_str(); // Add a line to beginning
                        self.buf_out = self // Remove last line
                            .buf_out
                            .lines()
                            .take(16)
                            .map(|s| s.to_string() + "\n")
                            .collect();
                    }
                }
            } else {
                break;
            }
        }
    }

    fn send_settings(&mut self) {
        if self.emu_use_khz {
            self.emu_tx.send(CtrlMSG::SetRate(self.emu_speed * 1000.));
        } else {
            self.emu_tx.send(CtrlMSG::SetRate(self.emu_speed));
        }
    }
}

impl eframe::App for TitoApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 60fps gui update when emulator is running
        if self.emu_running && self.emu_playing {
            ctx.request_repaint_after(std::time::Duration::from_secs(1 / 60))
        }
        self.msg_handler();
        self.send_settings();
        self.gui_main(ctx);
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: true,
        fullscreen: false,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_pos: None,
        initial_window_size: Some(egui::Vec2 { x: 800., y: 600. }),
        min_window_size: Some(egui::Vec2 { x: 800., y: 52. }),
        max_window_size: None,
        resizable: true,
        transparent: false,
        mouse_passthrough: false,
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        follow_system_theme: true,
        default_theme: eframe::Theme::Dark,
        run_and_return: false,
        event_loop_builder: None,
        shader_version: None,
        centered: true,
    };

    eframe::run_native(
        "Tito",
        native_options,
        Box::new(|cc| Box::new(TitoApp::new(cc))),
    );
}
