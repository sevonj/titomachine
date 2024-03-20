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

mod devices;
pub mod emu_debug;
mod perfmon;
#[cfg(test)]
mod tests;

use image::Rgba;
use libttktk::b91::B91;
use crate::emulator::cpu::GPR;

use self::cpu::CPU;
use self::devices::{Bus, Device};
use self::emu_debug::{CtrlMSG, ReplyMSG};
use self::perfmon::PerfMonitor;

mod cpu;

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
    bus: Bus,
    cpu: CPU,
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
            bus: Bus::new(),
            cpu: CPU::new(),
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
        emu.bus.crt.connect(tx_devcrt);
        emu.bus.kbd.connect(rx_devkbd, tx_devkbdreq);
        emu.bus.display.connect(tx_devdisplay);
        emu
    }

    pub fn update(&mut self) {
        self.timekeeper();
        self.check_mail();
        self.dev_update_slow();

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

    /// Fast update: every cpu tick
    fn dev_update(&mut self) {
        // Interrupts
        // if self.bus.pic.is_firing() {
        //     self.cpu.exception_irq(&mut self.bus);
        // }
    }

    /// Slow update: every frame or so
    fn dev_update_slow(&mut self) {
        self.bus.display.send();
        // if self.bus.display.interrupt {
        //     self.bus.pic.flag |= 0b_0100;
        // }
    }

    fn start(&mut self) {
        self.reload();
        self.cpu.init();
        self.running = true;
        self.t_last_update = None;
        self.bus.turn_on();
    }

    fn stop(&mut self) {
        self.t_last_update = None;
        self.running = false;
        self.playing = false;
        // Send framebuffer to avoid incomplete picture
        self.bus.display.send();
        self.bus.turn_off();
    }

    fn playpause(&mut self, p: bool) {
        self.t_last_update = None;
        self.playing = p;
        if p {
            self.bus.turn_on();
            // Perform one tick ignoring breakpoints, in case we're stopped on one.
            self.tick_ignore_breakpoints();
            thread::sleep(Duration::from_secs_f32(1. / self.tick_rate));
        } else {
            self.bus.turn_off()
        }
    }

    fn load_b91(&mut self, b91: B91) {
        self.stop();

        self.start_code = b91.code_segment.start;
        self.start_data = b91.data_segment.start;
        self.start_stack = b91.data_segment.end + 1;
        let _ = self.tx.send(ReplyMSG::SegmentOffsets(self.start_code, self.start_data, self.start_stack));

        // Load code segment
        let mut mem_off = b91.code_segment.start;
        for instruction in &b91.code_segment.content {
            self.bus.write(mem_off as u32, *instruction)
                .map_err(|err| println!("load_b91 writing code segment failed!\n{:?}", err))
                .ok();
            mem_off += 1;
        }

        // Load data segment
        let mut mem_off = b91.data_segment.start;
        for variable in &b91.data_segment.content {
            self.bus.write(mem_off as u32, *variable)
                .map_err(|err| println!("load_b91 writing data segment failed!\n{:?}", err))
                .ok();
            mem_off += 1;
        }

        // CPU registers
        self.cpu.init();
        self.cpu.debug_set_cu_pc(b91.code_segment.start as i32);
        self.cpu.debug_set_gpr(GPR::FP, b91.code_segment.end as i32);
        self.cpu.debug_set_gpr(GPR::SP, b91.data_segment.end as i32);

        // CPU Interrupt Vector Table
        for i in 0..=15 {
            if let Some(value) = b91.symbol_table.get(format!("__IVT_ENTRY_{i}__").as_str()) {
                self.cpu.debug_set_ivt(i, (*value).into())
            }
        }

        self.loaded_prog = Some(b91);
    }
    fn reset(&mut self) {
        self.stop();
        self.bus.reset();
        self.cpu = CPU::new();
        self.reload();
    }
    fn reload(&mut self) {
        self.stop();
        self.load_b91(self.loaded_prog.clone().unwrap());
    }

    fn clearmem(&mut self) {
        self.stop();
        self.bus.ram.reset();
        self.bus.display.reset();
    }

    /// Advance the emulator by one instruction.
    fn tick(&mut self) {
        self.dev_update();
        if self.cpu.halt {
            return;
        }
        if self.breakpoints_enabled {
            if self.breakpoints.contains(&(self.cpu.debug_get_cu_pc() as usize)) {
                self.playpause(false);
                return;
            }
        }
        self.cpu.tick(&mut self.bus);
    }

    /// Advance the emulator by one instruction. Ignore breakpoints,
    fn tick_ignore_breakpoints(&mut self) {
        self.dev_update();
        if self.cpu.halt {
            return;
        }
        self.cpu.tick(&mut self.bus);
    }
}
