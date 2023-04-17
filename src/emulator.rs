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
#[cfg(test)]
mod tests;

use image::Rgba;

use self::cpu::CPU;
use self::devices::{Bus, Device};
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
    tx_devdisplay: Sender<Vec<Rgba<u8>>>,
) {
    let mut emu = Emu::new(tx, rx, tx_devcrt, rx_devkbd, tx_devkbdreq, tx_devdisplay);
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
    t_delta: Duration,
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
        tx_devdisplay: Sender<Vec<Rgba<u8>>>,
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
            t_delta: Duration::ZERO,
            t_last_update: None,
            t_last_cpu_tick: None,
            perfmon: PerfMonitor::default(),
        };
        emu.bus.crt.connect(tx_devcrt);
        emu.bus.kbd.connect(rx_devkbd, tx_devkbdreq);
        emu.bus.display.connect(tx_devdisplay);
        emu
    }

    pub fn update(&mut self) {
        self.timekeeper();
        self.check_mail();
        self.dev_update_slow();

        let cyclecount = self.tick_rate as u32 / 60;
        let duration = Duration::from_secs_f32(1. / self.tick_rate) * cyclecount;
        match self.playing {
            // Playing
            true => {
                match self.tick_timer >= duration {
                    // Run
                    true => {
                        for _ in 0..cyclecount {
                            self.dev_update();
                            self.tick();
                        }
                        self.slow_checks();
                        self.tick_timer -= duration;
                    }
                    // Wait
                    false => {
                        if self.tick_rate < 10000000. {
                            thread::sleep(Duration::from_secs_f32(1. / self.tick_rate));
                        }
                    }
                }
            }
            // Not playing
            false => thread::sleep(Duration::from_secs_f32(1. / 60.)),
        }
    }

    /// Things that don't have to be done every cycle
    fn slow_checks(&mut self) {
        self.perfmon.update();
        self.t_last_cpu_tick = Some(Instant::now());
        if self.cpu.burn {
            self.stop();
        }
    }

    fn timekeeper(&mut self) {
        let now = Instant::now();
        self.t_delta;
        match self.t_last_update {
            Some(last) => self.t_delta = now - last,
            None => self.t_delta = Duration::ZERO,
        }
        self.t_last_update = Some(now);
        if self.playing {
            self.tick_timer += self.t_delta;
        }
        self.bus.pic.update_timer(self.t_delta);
    }

    /// Fast update: every cpu tick
    fn dev_update(&mut self) {
        // Interrupts
        if self.bus.pic.is_firing() {
            self.cpu.exception_irq(&mut self.bus);
        }
    }

    /// Slow update: every frame or so
    fn dev_update_slow(&mut self) {
        self.bus.display.send();
        if self.bus.display.interrupt {
            self.bus.pic.flag |= 0b_0100;
        }
    }

    fn check_mail(&mut self) {
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
                    CtrlMSG::Reset() => self.reset(),
                    CtrlMSG::LoadProg(fname) => self.loadprog(fname),
                    CtrlMSG::ClearMem => self.clearmem(),
                    // Settings
                    CtrlMSG::SetRate(rate) => self.tick_rate = rate,
                    CtrlMSG::SetTurbo(t) => self.turbo = t,
                    // Debug
                    CtrlMSG::GetState => self.debug_sendstate(),
                    CtrlMSG::GetMem(range) => self.debug_sendmem(range),
                }
            } else {
                break;
            }
        }
    }

    fn start(&mut self) {
        self.reload();
        self.cpu.init();
        self.running = true;
        self.t_last_update = None;
    }

    fn stop(&mut self) {
        self.t_last_update = None;
        self.running = false;
        self.playing = false;
        // Send framebuffer to avoid incomplete picture
        self.bus.display.send();
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
    fn reset(&mut self) {
        self.stop();
        self.bus.reset_devices();
        self.cpu = CPU::new();
        self.reload();
    }
    fn reload(&mut self) {
        self.stop();
        loader::load_program(&mut self.bus, &mut self.cpu, &self.loaded_prog);
    }

    fn clearmem(&mut self) {
        self.stop();
        self.bus.ram.reset();
        self.bus.display.reset();
    }

    fn tick(&mut self) {
        if self.cpu.halt {
            return;
        }
        self.cpu.tick(&mut self.bus);
    }
}
