use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::{
    ops::Range,
    time::{Duration, Instant},
};
pub mod instance;
use instance::*;

use self::instance::TTKInstance;
pub mod instructions;
pub mod loader;

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
    Display(Vec<i32>),
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
    instance: TTKInstance,

    tx: Sender<ReplyMSG>,
    rx: Receiver<CtrlMSG>,

    loaded_prog: String,

    playing: bool,
    tick_rate: f32,
    turbo: bool,

    t_last: Option<Instant>,
    since_last_tick: Duration,

    last_tick: Instant,
    last_tick_time: Duration,
}
impl Emu {
    pub fn default(tx: Sender<ReplyMSG>, rx: Receiver<CtrlMSG>) -> Self {
        Emu {
            tx,
            rx,
            instance: TTKInstance::default(),
            loaded_prog: String::new(),
            playing: false,
            tick_rate: 10.,
            turbo: false,

            // Time
            t_last: None,
            since_last_tick: Duration::ZERO,

            last_tick: Instant::now(),
            last_tick_time: Duration::from_micros(0),
        }
    }
    pub fn run(&mut self) {
        loop {
            // Time
            let t_tick = Duration::from_secs_f32(1. / self.tick_rate);
            let t_now = Instant::now();
            let t_delta;
            match self.t_last {
                Some(last) => t_delta = t_now - last,
                None => t_delta = Duration::ZERO,
            }
            self.t_last = Some(t_now);
            if self.playing {
                self.since_last_tick += t_delta;
            }

            // Messages
            loop {
                let msg = self.rx.try_recv().unwrap_or(CtrlMSG::None);
                match msg {
                    CtrlMSG::None => break,

                    // Playback control
                    CtrlMSG::Start => self.start(),
                    CtrlMSG::Stop => self.stop(),
                    CtrlMSG::PlayPause(p) => self.playpause(p),
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
                if self.turbo {
                    self.since_last_tick = Duration::ZERO;
                    self.tick();
                    continue;
                } else if self.since_last_tick >= t_tick {
                    self.since_last_tick -= t_tick;
                    self.tick();
                    continue;
                }
            }
            // Sleep longer if not running
            match self.instance.running && !self.instance.halted {
                true => thread::sleep(Duration::from_secs_f32(0.5 / self.tick_rate)),
                false => thread::sleep(Duration::from_secs_f32(1. / 60.)),
            }
        }
    }

    fn start(&mut self) {
        self.instance.cu_pc = 0;
        self.instance.cu_ir = 0;
        self.instance.cu_tr = 0;
        self.instance.cu_sr = 0;
        self.instance.running = true;
        self.instance.halted = false;
        self.t_last = None;
    }

    fn stop(&mut self) {
        self.t_last = None;
        self.instance.running = false;
    }

    fn playpause(&mut self, p: bool) {
        self.t_last = None;
        self.playing = p;
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
        if self.instance.cu_sr & SR_S != 0 {
            self.svc_handler();
        }
        if self.instance.cu_sr & SR_M != 0 {
            println!("Program Error: Forbidden memory address!");
            self.instance.halted = true;
        }
        if self.instance.cu_sr & SR_U != 0 {
            println!("Program Error: Unknown Instruction!");
            self.instance.halted = true;
        }
        if self.instance.cu_sr & SR_Z != 0 {
            println!("Program Error: Zero division!");
            self.instance.halted = true;
        }
        if self.instance.cu_sr & SR_O != 0 {
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
        retvec.push(self.instance.cu_pc);
        retvec.push(self.instance.cu_ir);
        retvec.push(self.instance.cu_tr);
        retvec.push(self.instance.cu_sr);
        for i in 0..8 {
            retvec.push(self.instance.gpr[i])
        }
        self.tx.send(ReplyMSG::Regs(retvec));
    }

    fn senddisp(&mut self) {
        let retvec = self.instance.memory[8192..8192 + 120 * 160].to_vec();
        self.tx.send(ReplyMSG::Display(retvec));
    }
}
