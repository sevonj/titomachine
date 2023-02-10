use std::error::Error;
use std::fs;

use std::ops::ControlFlow;
use std::{
    ops::Range,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
pub mod instance;
use eframe::glow::STENCIL_BACK_REF;
use instance::*;
pub mod instructions;
use instructions::*;
pub mod loader;
use loader::*;

pub enum CtrlMSG {
    None, // this feels like a pretty dumb solution.

    // Playback control
    Start,
    Stop,
    PlayPause(bool),
    Tick,

    // Dev
    In(i32),

    // Loader
    LoadProg(String),
    Clear,

    // Settings
    SetRate(f32),
    SetTurbo(bool),
    SetMemSize(usize),

    // Debug
    GetState,
    GetRegs,
    GetMem(Range<i32>),
    GetDisp,
}
pub enum ReplyMSG {
    None,

    State(EmuState),
    Regs(Vec<i32>),
    Mem(Vec<i32>),
    display(Vec<i32>),

    In,
    Out(i32),
}

pub struct EmuState {
    pub playing: bool,
    pub running: bool,
    pub halted: bool,
    pub speed_percent: f32,
}

pub struct Emu {
    instance: instance::TTKInstance,

    tx: mpsc::Sender<ReplyMSG>,
    rx: mpsc::Receiver<CtrlMSG>,

    loaded_prog: String,

    playing: bool,
    tick_rate: f32,
    turbo: bool,
    last_tick: Instant,
    last_tick_time: Duration,
}
impl Emu {
    pub fn default(tx: mpsc::Sender<ReplyMSG>, rx: mpsc::Receiver<CtrlMSG>) -> Self {
        let xx = 1..2;
        Emu {
            tx,
            rx,

            instance: instance::TTKInstance::default(),

            loaded_prog: String::new(),

            playing: false,
            tick_rate: 10.,
            turbo: false,
            last_tick: Instant::now(),
            last_tick_time: Duration::from_micros(0),
        }
    }
    pub fn run(&mut self) {
        loop {
            loop {
                let msg = self.rx.try_recv().unwrap_or(CtrlMSG::None);
                match msg {
                    CtrlMSG::None => {
                        break;
                    }

                    // Playback control
                    CtrlMSG::Start => self.start(),
                    CtrlMSG::Stop => self.stop(),
                    CtrlMSG::PlayPause(p) => self.playing = p,
                    CtrlMSG::Tick => self.tick(),

                    // Dev
                    CtrlMSG::In(input) => self.input_handler(input),

                    // Loader
                    CtrlMSG::Clear => loader::clear(&mut self.instance),
                    CtrlMSG::LoadProg(fname) => self.loadprog(fname),

                    // Settings
                    CtrlMSG::SetRate(rate) => self.tick_rate = rate,
                    CtrlMSG::SetTurbo(t) => self.turbo = t,
                    CtrlMSG::SetMemSize(size) => loader::setmemsize(&mut self.instance, size),

                    // Debug
                    CtrlMSG::GetState => self.sendstate(),
                    CtrlMSG::GetMem(range) => self.sendmem(range),
                    CtrlMSG::GetRegs => self.sendregs(),
                    CtrlMSG::GetDisp => self.senddisp(),
                }
            }

            if self.playing {
                if self.turbo
                    || self.last_tick.elapsed() > Duration::from_secs_f32(1. / self.tick_rate)
                {
                    self.tick();
                    continue;
                }
            }
            if self.instance.running && !self.instance.halted {
                if self.turbo {
                    thread::sleep(Duration::from_micros(1));
                } else {
                    thread::sleep(Duration::from_secs_f32(0.5 / self.tick_rate));
                }
                continue;
            }
            thread::sleep(Duration::from_millis(16));
        }
    }

    fn start(&mut self) {
        self.instance.pc = 0;
        self.instance.ir = 0;
        self.instance.tr = 0;
        self.instance.sr = SR_DEFAULT;
        self.instance.running = true;
        self.instance.halted = false;
    }

    fn stop(&mut self) {
        self.instance.running = false;
    }

    fn loadprog(&mut self, prog: String) {
        self.instance.running = false;
        loader::clear(&mut self.instance);
        self.loaded_prog = prog;
        loader::load_program(&self.loaded_prog, &mut self.instance);
    }

    fn tick(&mut self) {
        if !self.instance.running {
            return;
        }
        if self.instance.halted {
            return;
        }
        if self.instance.waiting_for_io {
            return;
        }
        self.last_tick_time = Instant::now() - self.last_tick;
        self.last_tick = Instant::now();
        self.exec();
        self.sr_handler();

        // Dunno why, but being halted causes the emulator to max out cpu on host machine.
        // So this stops the emulation. It's not like it needs to run after halt anyway.
        if self.instance.halted {
            self.instance.running = false;
            self.playing = false;
        }
    }

    // Check for anomalies in state registers
    fn sr_handler(&mut self) {
        if self.instance.sr & SR_S != 0 {
            self.svc_handler();
        }
        if self.instance.sr & SR_M != 0 {
            println!("Program Error: Forbidden memory address!");
            self.instance.halted = true;
        }
        if self.instance.sr & SR_U != 0 {
            println!("Program Error: Unknown Instruction!");
            self.instance.halted = true;
        }
        if self.instance.sr & SR_Z != 0 {
            println!("Program Error: Zero division!");
            self.instance.halted = true;
        }
        if self.instance.sr & SR_O != 0 {
            println!("Program Error: Overflow!");
            self.instance.halted = true;
        }
    }

    fn sendstate(&mut self) {
        let speed = 100. / (self.last_tick_time.as_secs_f32() * self.tick_rate);
        self.tx.send(ReplyMSG::State(EmuState {
            playing: self.playing,
            running: self.instance.running,
            halted: self.instance.halted,
            speed_percent: speed,
        }));
    }

    fn sendmem(&mut self, range: Range<i32>) {
        let mut retvec: Vec<i32> = Vec::with_capacity(range.len());
        for i in range.clone() {
            if i >= self.instance.memory.len() as i32 {
                break;
            }
            retvec.push(self.instance.memory[i as usize]);
        }
        self.tx.send(ReplyMSG::Mem(retvec));
    }

    fn sendregs(&mut self) {
        let mut retvec: Vec<i32> = Vec::with_capacity(12);
        retvec.push(self.instance.pc);
        retvec.push(self.instance.ir);
        retvec.push(self.instance.tr);
        retvec.push(self.instance.sr);
        for i in 0..8 {
            retvec.push(self.instance.gpr[i])
        }
        self.tx.send(ReplyMSG::Regs(retvec));
    }

    fn senddisp(&mut self) {
        let retvec = self.instance.memory[8192..8192 + 120 * 160].to_vec();
        self.tx.send(ReplyMSG::display(retvec));
    }
}
