///
/// main.rs
///
/// High level project structure looks like this:
///
/// Main/
///     Editor/
///         Most of the editor is just GUI code, so this dir is pretty empty.
///         Compiler lies here.
///
///     Emulator/
///         The emulator runs on a separate thread.
///         This is the largest component.
///         Read the top comment at src/emulator.rs for structure.
///
///     GUI/
///         Contains gui code, which at times is rather messy.
///         Further divided into 3 files: Editor GUI, Emulator GUI, and File actions.
///
///
#[macro_use]
extern crate num_derive;
use std::{env::current_dir, path::PathBuf, sync::mpsc, thread};
pub mod editor;
pub mod emulator;
pub mod gui;
use editor::{Editor, EditorSettings};

use emulator::emu_debug::{CtrlMSG, DebugRegs, ReplyMSG};
use gui::{
    file_actions::FileStatus,
    gui_emulator::gui_devices::{display::GUIDevDisplay, legacy_io::GUIDevLegacyIO, GUIDevice},
    Base, GuiMode,
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
// TODO: Cleanup
pub struct TitoApp {
    // File, Status
    workdir: PathBuf,
    #[serde(skip)]
    filestatus: FileStatus,
    editorsettings: EditorSettings,
    #[serde(skip)]
    editor: Editor,

    // Devices
    #[serde(skip)]
    dev_legacyio: GUIDevLegacyIO,
    #[serde(skip)]
    dev_display: GUIDevDisplay,

    // Emulator
    #[serde(skip)]
    tx_ctrl: mpsc::Sender<CtrlMSG>,
    #[serde(skip)]
    rx_reply: mpsc::Receiver<ReplyMSG>,
    current_prog: String,

    // Emu status, settings
    emu_running: bool,
    emu_halted: bool,
    emu_playing: bool,
    emu_cpuspeedmul: FreqMagnitude,
    emu_speed: f32,
    #[serde(skip)]
    emu_turbo: bool,
    #[serde(skip)]
    emu_achieved_speed: f32,
    #[serde(skip)]
    emu_regs: DebugRegs,
    #[serde(skip)]
    gui_memview: Vec<i32>, // Cached partial memory for gui
    #[serde(skip)]
    gui_memview_off: u32, // Start offset
    #[serde(skip)]
    gui_memview_len: u32, // Size of cache
    #[serde(skip)]
    emu_mem_len: usize, // Size of cache
    #[serde(skip)]
    gui_memview_scroll: f32,

    // GUI settings
    #[serde(skip)]
    guimode: GuiMode,
    emugui_display: bool,
    emugui_follow_pc: bool,
    mem_adr_base: Base,
    mem_val_base: Base,
    regs_base: Base,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
pub(crate) enum FreqMagnitude {
    Hz,
    KHz,
    MHz,
}

impl Default for TitoApp {
    fn default() -> Self {
        let (tx_control, rx_control) = mpsc::channel();
        let (tx_reply, rx_reply) = mpsc::channel();
        let (tx_devcrt, rx_devcrt) = mpsc::channel();
        let (tx_devkbd, rx_devkbd) = mpsc::channel();
        let (tx_devkbdreq, rx_devkbdreq) = mpsc::channel();
        let (tx_devdisplay, rx_devdisplay) = mpsc::channel();

        let dev_legacyio = GUIDevLegacyIO::new(rx_devcrt, tx_devkbd, rx_devkbdreq);
        let dev_display = GUIDevDisplay::new(rx_devdisplay);

        thread::spawn(move || {
            emulator::run(
                tx_reply,
                rx_control,
                tx_devcrt,
                rx_devkbd,
                tx_devkbdreq,
                tx_devdisplay,
            );
        });
        TitoApp {
            workdir: current_dir().unwrap(),
            filestatus: FileStatus::default(),
            editorsettings: EditorSettings::default(),
            editor: Editor::default(),
            // Emulator
            tx_ctrl: tx_control,
            rx_reply,
            dev_legacyio,
            dev_display,
            current_prog: String::new(),

            emu_running: false,
            emu_halted: false,
            emu_playing: false,
            emu_speed: 10.,
            emu_achieved_speed: 0.,
            emu_cpuspeedmul: FreqMagnitude::Hz,
            emu_turbo: false,
            emu_regs: DebugRegs::default(),
            emu_mem_len: 0,
            gui_memview: vec![7; 16],
            gui_memview_off: 0,
            gui_memview_len: 16,
            gui_memview_scroll: 0.,

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
            if let Ok(msg) = self.rx_reply.try_recv() {
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
                }
            } else {
                break;
            }
        }
    }

    fn send_settings(&mut self) {
        let speed = match self.emu_cpuspeedmul {
            FreqMagnitude::Hz => self.emu_speed,
            FreqMagnitude::KHz => self.emu_speed * 1000.,
            FreqMagnitude::MHz => self.emu_speed * 1000000.,
        };
        match self.tx_ctrl.send(CtrlMSG::SetRate(speed)) {
            Ok(_) => (),
            Err(_) => todo!(),
        }
    }
    fn stop_emulation(&mut self) {
        self.emu_running = false;
        self.dev_legacyio.clear_kbd();
        self.tx_ctrl.send(CtrlMSG::PlaybackStop);
    }

    fn update_devices(&mut self) {
        self.dev_legacyio.update();
        self.dev_display.update();
    }
}

impl eframe::App for TitoApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 60fps gui update when emulator is running
        self.update_devices();

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
