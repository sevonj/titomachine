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
use std::env;
use std::{path::PathBuf, sync::mpsc, thread};
pub mod editor;
pub mod emulator;
pub mod gui;
use editor::*;
use egui_extras::RetainedImage;
use emulator::{CtrlMSG, Emu, ReplyMSG};
use gui::{Base, GuiMode};
use image::ImageBuffer;
use image::Rgba;

const mem_display_size: usize = 512;

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
    emu_play_speed: f32,
    #[serde(skip)]
    emu_achieved_speed: f32,
    #[serde(skip)]
    emu_turbo: bool,
    #[serde(skip)]
    emu_registers: Vec<i32>, // Cached registers for gui
    #[serde(skip)]
    emu_memory: Vec<i32>, // Cached partial memory for gui
    #[serde(skip)]
    emu_memory_off: i32, // Start offset
    #[serde(skip)]
    emu_memory_len: i32, // Size of cache
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
    memview_adr_base: Base,
    memview_val_base: Base,
    register_val_base: Base,
}

impl Default for TitoApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        thread::spawn(move || {
            let mut emu = Emu::default(tx2, rx);
            emu.run();
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
            emu_play_speed: 10.,
            emu_achieved_speed: 0.,
            emu_use_khz: false,
            emu_turbo: false,
            emu_registers: vec![0; 12],
            emu_memory: vec![7; mem_display_size],
            emu_memory_off: 0,
            emu_memory_len: mem_display_size as i32,
            emu_waiting_for_in: false,
            emu_displayimage: None,
            emu_displaybuffer: None,
            emu_dispvec: vec![0; 120 * 160],

            // GUI
            guimode: GuiMode::Editor,
            emugui_display: false,
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
        
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
    fn msg_handler(&mut self) {
        loop {
            match self.emu_rx.try_recv().unwrap_or(ReplyMSG::None) {
                // Emulator State
                ReplyMSG::State(st) => {
                    self.emu_running = st.running;
                    self.emu_halted = st.halted;
                    self.emu_playing = st.playing;
                    self.emu_achieved_speed = st.speed_percent;
                }
                ReplyMSG::Regs(vec) => self.emu_registers = vec,
                ReplyMSG::Mem(vec) => self.emu_memory = vec,
                ReplyMSG::display(vec) => {
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
                ReplyMSG::None => {
                    break;
                }
            }
        }
    }
}

impl eframe::App for TitoApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.msg_handler();
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
        "TiToMachine",
        native_options,
        Box::new(|cc| Box::new(TitoApp::new(cc))),
    );
}
