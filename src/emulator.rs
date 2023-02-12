use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::{
    ops::Range,
    time::{Duration, Instant},
};
pub mod instance;
mod performance_monitor;
use instance::*;

use self::instance::TTKInstance;
use self::performance_monitor::PerfMonitor;
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
    MemSize(usize),
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

    t_last_update: Option<Instant>,
    t_tick_timer: Duration,
    t_last_tick: Option<Instant>,
    perfmon: PerfMonitor,
    interrupt_timer: Option<Duration>,
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
            t_last_update: None,
            t_tick_timer: Duration::ZERO,
            t_last_tick: None,
            perfmon: PerfMonitor::default(),
            interrupt_timer: None,
        }
    }
    pub fn run(&mut self) {
        loop {
            // Time
            let tick_time = Duration::from_secs_f32(1. / self.tick_rate);
            let t_now = Instant::now();
            let t_delta;
            match self.t_last_update {
                Some(last) => t_delta = t_now - last,
                None => t_delta = Duration::ZERO,
            }
            self.t_last_update = Some(t_now);
            if self.playing {
                self.t_tick_timer += t_delta;
            }
            self.perfmon.set_rate(self.tick_rate);
            if let Some(d) = self.interrupt_timer {
                self.interrupt_timer = Some(d - t_delta);
                if d == Duration::ZERO {
                    self.interrupt_timer = None;
                    self.instance.cu_sr |= SR_I;
                }
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
                    self.t_tick_timer = Duration::ZERO;
                    self.tick();
                    if let Some(t) = self.t_last_tick {
                        self.perfmon.add_sample(Instant::now() - t);
                    }
                    self.t_last_tick = Some(Instant::now());
                    continue;
                } else if self.t_tick_timer >= tick_time {
                    self.t_tick_timer -= tick_time;
                    self.tick();
                    if let Some(t) = self.t_last_tick {
                        self.perfmon.add_sample(Instant::now() - t);
                    }
                    self.t_last_tick = Some(Instant::now());
                    continue;
                }
            }
            // Sleep longer if not running
            match self.instance.running && !self.instance.halt {
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
        self.instance.halt = false;
        self.t_last_update = None;
    }

    fn stop(&mut self) {
        self.t_last_update = None;
        self.instance.running = false;
    }

    fn playpause(&mut self, p: bool) {
        self.t_last_update = None;
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
        if self.instance.halt {
            return;
        }
        if self.instance.waiting_for_io {
            return;
        }
        self.exec();
        self.sr_handler();

        // Dunno why, but being halted causes the emulator to max out cpu on host machine.
        // So this stops the emulation. It's not like it needs to run after halt anyway.
        if self.instance.halt {
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
            self.instance.halt = true;
        }
        if self.instance.cu_sr & SR_U != 0 {
            println!("Program Error: Unknown Instruction!");
            self.instance.halt = true;
        }
        if self.instance.cu_sr & SR_Z != 0 {
            println!("Program Error: Zero division!");
            self.instance.halt = true;
        }
        if self.instance.cu_sr & SR_O != 0 {
            println!("Program Error: Overflow!");
            self.instance.halt = true;
        }
    }

    fn sendstate(&mut self) {
        self.tx.send(ReplyMSG::State(EmuState {
            playing: self.playing,
            running: self.instance.running,
            halted: self.instance.halt,
            speed_percent: self.perfmon.get_percent(),
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
        self.tx.send(ReplyMSG::MemSize(self.instance.memory.len()));
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
