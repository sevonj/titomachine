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
extern crate num_derive;

use crate::gui::gui_emulator::memoryview::MemoryView;
use std::{env::current_dir, path::PathBuf, sync::mpsc, thread};
use egui::{Context, Vec2, ViewportBuilder};
use egui_extras::install_image_loaders;

pub mod config;
pub mod editor;
pub mod emulator;
pub mod gui;

use editor::Editor;

use emulator::emu_debug::{CtrlMSG, DebugRegs, ReplyMSG};
use gui::{
    gui_editor::file_actions::FileStatus,
    gui_emulator::gui_devices::{GUIDevice, legacy_io::GUIDevLegacyIO},
    GuiMode,
};
use crate::config::Config;
use crate::gui::gui_emulator::cpuview::CPUView;
use crate::gui::gui_emulator::graphicsview::GraphicsView;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
// TODO: Cleanup
pub struct TitoApp {
    config: Config,

    #[serde(skip)] filestatus: FileStatus,
    #[serde(skip)] editor: Editor,

    // Devices
    #[serde(skip)] dev_legacyio: GUIDevLegacyIO,
    //#[serde(skip)] dev_display: GUIDevDisplay,

    // Emulator
    #[serde(skip)] tx_ctrl: mpsc::Sender<CtrlMSG>,
    #[serde(skip)] rx_reply: mpsc::Receiver<ReplyMSG>,

    // Emu status, settings
    #[serde(skip)] emu_running: bool,
    #[serde(skip)] emu_halted: bool,
    #[serde(skip)] emu_playing: bool,

    #[serde(skip)] emu_turbo: bool,
    #[serde(skip)] emu_achieved_speed: f32,
    #[serde(skip)] graphicsview: GraphicsView,
    #[serde(skip)] memoryview: MemoryView,
    #[serde(skip)] cpuview: CPUView,

    // GUI settings
    #[serde(skip)] guimode: GuiMode,

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
        //let dev_display = GUIDevDisplay::new(rx_devdisplay);

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
            config: Config::default(),
            filestatus: FileStatus::default(),
            editor: Editor::default(),
            // Emulator
            tx_ctrl: tx_control,
            rx_reply,
            dev_legacyio,
            //dev_display,

            emu_running: false,
            emu_halted: false,
            emu_playing: false,
            emu_achieved_speed: 0.,
            emu_turbo: false,
            graphicsview: GraphicsView::new(rx_devdisplay),
            memoryview: MemoryView::new(),
            cpuview: CPUView::new(),

            // GUI
            guimode: GuiMode::Editor,
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
                    // Todo: Regs message could be merged into State
                    // Emulator State
                    ReplyMSG::State(st) => {
                        self.emu_running = st.running;
                        self.emu_halted = st.halted;
                        self.cpuview.cpu_halt = st.halted;
                        self.emu_playing = st.playing;
                        self.emu_achieved_speed = st.speed_percent;
                        self.memoryview.is_playing = st.running && st.playing && !st.halted;
                    }
                    ReplyMSG::Regs(regs) => {
                        self.memoryview.cpu_pc = regs.pc as usize;
                        self.memoryview.cpu_sp = regs.gpr[6] as usize;
                        self.memoryview.cpu_fp = regs.gpr[7] as usize;

                        self.cpuview.cpu_cu_pc = regs.pc;
                        self.cpuview.cpu_gpr_r0 = regs.gpr[0];
                        self.cpuview.cpu_gpr_r1 = regs.gpr[1];
                        self.cpuview.cpu_gpr_r2 = regs.gpr[2];
                        self.cpuview.cpu_gpr_r3 = regs.gpr[3];
                        self.cpuview.cpu_gpr_r4 = regs.gpr[4];
                        self.cpuview.cpu_gpr_r5 = regs.gpr[5];
                        self.cpuview.cpu_gpr_sp = regs.gpr[6];
                        self.cpuview.cpu_gpr_fp = regs.gpr[7];
                        self.cpuview.cpu_cu_sr = regs.sr;
                    }
                    ReplyMSG::Mem(vec) => {
                        self.memoryview.set_view_cache(self.memoryview.get_view_cache_start(), vec)
                    }
                    ReplyMSG::SegmentOffsets(start_code, start_data, start_stack) => {
                        self.memoryview.start_code = start_code;
                        self.memoryview.start_data = start_data;
                        self.memoryview.start_stack = start_stack;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn send_settings(&mut self) {
        let speed = match self.config.emu_cpuspeedmul {
            FreqMagnitude::Hz => self.config.emu_speed,
            FreqMagnitude::KHz => self.config.emu_speed * 1000.,
            FreqMagnitude::MHz => self.config.emu_speed * 1000000.,
        };
        let _ = self.tx_ctrl.send(CtrlMSG::SetRate(speed));
    }

    fn stop_emulation(&mut self) {
        self.emu_running = false;
        self.dev_legacyio.clear_kbd();
        let _ = self.tx_ctrl.send(CtrlMSG::PlaybackStop);
    }

    fn update_devices(&mut self) {
        self.dev_legacyio.update();
        //self.dev_display.update();
    }
}

impl eframe::App for TitoApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        install_image_loaders(ctx);

        // 60fps gui update when emulator is running
        self.update_devices();

        if self.emu_running && self.emu_playing {
            ctx.request_repaint_after(std::time::Duration::from_secs(1 / 60))
        }
        self.msg_handler();
        self.send_settings();
        self.gui_main(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id("fi.sevonj.titomachine")
            .with_inner_size(Vec2 { x: 800., y: 600. })
            .with_min_inner_size(Vec2 { x: 800., y: 52. }),
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::default(),
        follow_system_theme: true,
        default_theme: eframe::Theme::Dark,
        run_and_return: false,
        event_loop_builder: None,
        window_builder: None,
        shader_version: None,
        centered: true,
        persist_window: false,
    };

    let _ = eframe::run_native(
        "Tito",
        native_options,
        Box::new(|cc| Box::new(TitoApp::new(cc))),
    );
}