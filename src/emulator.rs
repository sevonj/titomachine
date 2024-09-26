use std::collections::HashSet;
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

pub mod emu_debug;
mod perfmon;

use image::Rgba;
use tito_core::b91::B91;
use tito_core::machine::Machine;

use self::emu_debug::{CtrlMSG, ReplyMSG};
use self::perfmon::PerfMonitor;
use tito_core::machine::devices::Device;

// There has to be a cleaner way to pass the channels.
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
    //bus: Bus,
    //cpu: CPU,
    machine: Machine,
    tx: Sender<ReplyMSG>,
    rx: Receiver<CtrlMSG>,
    loaded_prog: Option<B91>,
    start_code: usize,
    start_data: usize,
    start_stack: usize,
    running: bool,
    playing: bool,
    tick_rate: f32,
    turbo: bool,
    tick_timer: Duration,
    t_delta: Duration,
    t_last_update: Option<Instant>,
    t_last_cpu_tick: Option<Instant>,
    perfmon: PerfMonitor,
    breakpoints_enabled: bool,
    breakpoints: HashSet<usize>,
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
            //bus: Bus::new(),
            //cpu: CPU::new(),
            machine: Machine::new(),
            tx,
            rx,
            loaded_prog: None,
            start_code: 0,
            start_data: 0,
            start_stack: 0,
            running: false,
            playing: false,
            tick_rate: 10.,
            turbo: false,
            tick_timer: Duration::ZERO,
            t_delta: Duration::ZERO,
            t_last_update: None,
            t_last_cpu_tick: None,
            perfmon: PerfMonitor::default(),
            breakpoints_enabled: false,
            breakpoints: HashSet::new(),
        };
        //emu.bus.crt.connect(tx_devcrt);
        //emu.bus.kbd.connect(rx_devkbd, tx_devkbdreq);
        //emu.bus.display.connect(tx_devdisplay);
        emu
    }

    pub fn update(&mut self) {
        self.timekeeper();
        self.check_mail();

        let cyclecount = self.tick_rate as u32 / 60 + 1;
        if self.playing {
            if self.time_to_run(cyclecount) {
                for _ in 0..cyclecount {
                    self.tick();
                }
                self.slow_checks();
            } else {
                if self.tick_rate < 10000000. {
                    thread::sleep(Duration::from_secs_f32(1. / self.tick_rate));
                }
            }
        } else {
            thread::sleep(Duration::from_secs_f32(1. / 60.))
        }
    }

    fn time_to_run(&mut self, cyclecount: u32) -> bool {
        let duration = Duration::from_secs_f32(1. / self.tick_rate) * cyclecount;
        match self.tick_timer >= duration {
            true => {
                self.tick_timer -= duration;
                true
            }
            false => false,
        }
    }

    /// When user clicks step button
    pub fn manual_tick(&mut self) {
        self.tick_ignore_breakpoints();
        self.slow_checks();
    }

    /// Things that don't have to be done every cycle
    fn slow_checks(&mut self) {
        self.perfmon.update();
        self.t_last_cpu_tick = Some(Instant::now());
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
        // self.bus.pic.update_timer(self.t_delta);
    }

    fn start(&mut self) {
        self.reload_program();
        self.machine.reset_soft();
        self.running = true;
        self.t_last_update = None;
    }

    fn stop(&mut self) {
        self.t_last_update = None;
        self.running = false;
        self.playing = false;
        // Send framebuffer to avoid incomplete picture
        //self.bus.display.send();
        //self.bus.turn_off();
    }

    fn playpause(&mut self, p: bool) {
        self.t_last_update = None;
        self.playing = p;
        //self.bus.set_pause(p);
        if p {
            // Perform one tick ignoring breakpoints, in case we're stopped on one.
            self.tick_ignore_breakpoints();
            thread::sleep(Duration::from_secs_f32(1. / self.tick_rate));
        }
    }

    fn load_b91(&mut self, b91: B91) {
        self.stop();

        self.start_code = b91.code_segment.start as usize;
        self.start_data = b91.data_segment.start as usize;
        self.start_stack = (b91.data_segment.end + 1) as usize;
        let _ = self.tx.send(ReplyMSG::SegmentOffsets(
            self.start_code,
            self.start_data,
            self.start_stack,
        ));

        // Load code segment
        let mut mem_off = b91.code_segment.start;
        for instruction in &b91.code_segment.content {
            self.machine
                .debug_write_mem(mem_off as u32, *instruction)
                .map_err(|err| println!("load_b91 writing code segment failed!\n{:?}", err))
                .ok();
            mem_off += 1;
        }

        // Load data segment
        let mut mem_off = b91.data_segment.start;
        for variable in &b91.data_segment.content {
            self.machine
                .debug_write_mem(mem_off as u32, *variable)
                .map_err(|err| println!("load_b91 writing data segment failed!\n{:?}", err))
                .ok();
            mem_off += 1;
        }

        // CPU registers
        self.machine.reset_soft();
        self.machine.debug_set_cpu_pc(b91.code_segment.start as i32);
        self.machine.debug_set_cpu_fp(b91.code_segment.end as i32);
        self.machine.debug_set_cpu_sp(b91.data_segment.end as i32);

        self.loaded_prog = Some(b91);
    }
    fn reset(&mut self) {
        self.stop();
        self.machine.reset();
        self.reload_program();
    }
    fn reload_program(&mut self) {
        self.stop();
        self.machine.load_b91(self.loaded_prog.clone().unwrap());
    }

    /// Advance the machine by one instruction.
    fn tick(&mut self) {
        if self.breakpoints_enabled {
            if self
                .breakpoints
                .contains(&(self.machine.debug_get_cpu_pc() as usize))
            {
                self.playpause(false);
                return;
            }
        }
        self.machine.tick();
    }

    /// Advance the machine by one instruction. Ignore breakpoints.
    fn tick_ignore_breakpoints(&mut self) {
        self.machine.tick();
    }
}
