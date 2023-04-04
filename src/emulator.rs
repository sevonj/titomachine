///
/// emulator.rs
///
/// Struct structure:
/// Emu/
///     CPU/
///     Bus/
///         DevRAM
///         DevKBD
///         ...
///     Misc..
///
/// The Bus instance is passed to CPU at every tick().
///
/// File structure:
///     CPU:
///         The CPU is divided into further parts. To be reorganzed.
///         instructions.rs:
///         mmu.rs
///         cpu_debug.rs
///
///     Devices:
///         devices.rs contains the Bus struct, which is passed to the CPU.
///         Every device besides is within the Bus instance.
///         Devices directory contains every device.
///     
///     emu_debug:
///         Communicates with the gui.
///
///     loader:
///         Loads compiled program to memory
///
///     perfmon:
///         Performance monitor
///
///
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
mod devices;
pub mod emu_debug;
mod perfmon;

use self::cpu::CPU;
use self::devices::{Bus, MMIO};
use self::emu_debug::{CtrlMSG, ReplyMSG};
use self::perfmon::PerfMonitor;
mod cpu;
mod loader;

pub fn run(
    tx: Sender<ReplyMSG>,
    rx: Receiver<CtrlMSG>,
    tx_devcrt: Sender<i32>,
    rx_devkbd: Receiver<i32>,
    tx_devkbdreq: Sender<()>,
) {
    let mut emu = Emu::new(tx, rx, tx_devcrt, rx_devkbd, tx_devkbdreq);
    loop {
        emu.update();
    }
}

pub struct Emu {
    bus: Bus,
    cpu: CPU,
    tx: Sender<ReplyMSG>,
    rx: Receiver<CtrlMSG>,
    loaded_prog: String,
    running: bool,
    playing: bool,
    tick_rate: f32,
    turbo: bool,
    tick_timer: Duration,
    mail_timer: Duration,
    t_last_update: Option<Instant>,
    t_last_cpu_tick: Option<Instant>,
    perfmon: PerfMonitor,
}

impl Emu {
    pub fn new(
        tx: Sender<ReplyMSG>,
        rx: Receiver<CtrlMSG>,
        tx_devcrt: Sender<i32>,
        rx_devkbd: Receiver<i32>,
        tx_devkbdreq: Sender<()>,
    ) -> Self {
        let mut emu = Emu {
            bus: Bus::new(),
            cpu: CPU::new(),
            tx,
            rx,
            loaded_prog: String::new(),
            running: false,
            playing: false,
            tick_rate: 10.,
            turbo: false,
            tick_timer: Duration::ZERO,
            mail_timer: Duration::ZERO,
            t_last_update: None,
            t_last_cpu_tick: None,
            perfmon: PerfMonitor::default(),
        };
        emu.bus.crt.connect(tx_devcrt);
        emu.bus.kbd.connect(rx_devkbd, tx_devkbdreq);
        emu
    }

    pub fn update(&mut self) {
        self.timekeeper();
        self.do_devices();
        self.check_mail();
        if self.playing {
            let tick_time = Duration::from_secs_f32(1. / self.tick_rate);
            if self.turbo {
                // Turbomode: No limits!
                self.tick_timer = Duration::ZERO;
                self.tick();
            } else {
                // Normomode: Wait for tick timer
                if self.tick_timer >= tick_time {
                    self.tick_timer -= tick_time;
                    self.tick();
                } else {
                    // If no tick, sleep
                    thread::sleep(Duration::from_secs_f32(0.5 / self.tick_rate))
                }
            }
        } else {
            // Sleep longer when not playing
            thread::sleep(Duration::from_secs_f32(1. / 60.));
        }
    }

    fn timekeeper(&mut self) {
        let now = Instant::now();
        let delta;
        match self.t_last_update {
            Some(last) => delta = now - last,
            None => delta = Duration::ZERO,
        }
        self.t_last_update = Some(now);
        if self.playing {
            self.tick_timer += delta;
        }
        self.bus.pic.update_timer(delta);
        self.mail_timer += delta;
    }

    fn do_devices(&mut self) {
        self.bus.pic.update_status();
        self.cpu.set_sr_i(self.bus.pic.firing)
    }

    fn check_mail(&mut self) {
        // check mail less often
        if self.mail_timer < Duration::from_secs_f32(1. / 60.) {
            return;
        }
        self.mail_timer = Duration::ZERO;
        // Loop until there are no messages, because messages may arrive faster than this is called.
        loop {
            if let Ok(msg) = self.rx.try_recv() {
                match msg {
                    // Playback control
                    CtrlMSG::PlaybackStart => self.start(),
                    CtrlMSG::PlaybackStop => self.stop(),
                    CtrlMSG::PlaybackPlayPause(p) => self.playpause(p),
                    CtrlMSG::PlaybackTick => self.tick(),
                    // Loader
                    CtrlMSG::LoadProg(fname) => self.loadprog(fname),
                    CtrlMSG::ClearMem => self.clearmem(),
                    // Settings
                    CtrlMSG::SetRate(rate) => self.tick_rate = rate,
                    CtrlMSG::SetTurbo(t) => self.turbo = t,
                    // Debug
                    CtrlMSG::GetState => self.debug_sendstate(),
                    CtrlMSG::GetMem(range) => self.debug_sendmem(range),
                    CtrlMSG::GetRegs => self.debug_sendregs(),
                    CtrlMSG::GetDisp => self.debug_senddisp(),
                }
            } else {
                break;
            }
        }
    }

    fn start(&mut self) {
        self.reload();
        self.cpu.debug_clear_cu();
        self.running = true;
        self.cpu.debug_set_halt(false);
        self.cpu.debug_clear_fire();
        self.t_last_update = None;
    }

    fn stop(&mut self) {
        self.t_last_update = None;
        self.running = false;
    }

    fn playpause(&mut self, p: bool) {
        self.t_last_update = None;
        self.playing = p;
    }

    fn loadprog(&mut self, prog: String) {
        self.stop();
        self.loaded_prog = prog;
        loader::load_program(&mut self.bus, &mut self.cpu, &self.loaded_prog);
    }
    fn reload(&mut self) {
        self.bus.reset_devices();
        loader::load_program(&mut self.bus, &mut self.cpu, &self.loaded_prog);
    }

    fn clearmem(&mut self) {
        self.stop();
        self.bus.ram.clear();
        self.bus.display.clear();
    }

    fn tick(&mut self) {
        if !self.running {
            return;
        }
        self.perfmon.tick();
        if self.cpu.debug_get_halt() {
            return;
        }
        self.t_last_cpu_tick = Some(Instant::now());
        self.cpu.tick(&mut self.bus);

        if self.cpu.debug_is_on_fire() {
            self.playing = false;
            self.running = false;
        }
    }
}
