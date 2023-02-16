use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
pub mod emu_debug;
mod perfmon;
use self::cpu::{CPU, SR_I, SR_M, SR_O, SR_S, SR_U, SR_Z};
use self::emu_debug::{CtrlMSG, ReplyMSG};
use self::perfmon::PerfMonitor;
mod cpu;
mod loader;

pub struct Emu {
    cpu: CPU,
    ctlr_tx: Sender<ReplyMSG>,
    ctrl_rx: Receiver<CtrlMSG>,
    loaded_prog: String,
    playing: bool,
    tick_rate: f32,
    turbo: bool,
    tick_timer: Duration,
    t_last_update: Option<Instant>,
    t_last_cpu_tick: Option<Instant>,
    perfmon: PerfMonitor,
    //interrupt_timer: Option<Duration>,
}

impl Emu {
    pub fn default(tx: Sender<ReplyMSG>, rx: Receiver<CtrlMSG>) -> Self {
        Emu {
            cpu: CPU::new(),
            ctlr_tx: tx,
            ctrl_rx: rx,
            loaded_prog: String::new(),
            playing: false,
            tick_rate: 10.,
            turbo: false,
            tick_timer: Duration::ZERO,
            t_last_update: None,
            t_last_cpu_tick: None,
            perfmon: PerfMonitor::default(),
            //interrupt_timer: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.timekeeper();
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
                    }
                    thread::sleep(Duration::from_secs_f32(0.5 / self.tick_rate))
                }
            } else {
                // Sleep longer when not playing
                thread::sleep(Duration::from_secs_f32(1. / 60.));
            }
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
        self.perfmon.set_rate(self.tick_rate);
        //if let Some(d) = self.interrupt_timer {
        //    self.interrupt_timer = Some(d - delta);
        //    if d == Duration::ZERO {
        //        self.interrupt_timer = None;
        //        self.cpu.cu_sr |= SR_I;
        //    }
        //}
    }

    fn check_mail(&mut self) {
        // Loop until there are no messages, because messages may
        // come faster than update.
        loop {
            if let Ok(msg) = self.ctrl_rx.try_recv() {
                match msg {
                    // Playback control
                    CtrlMSG::Start => self.start(),
                    CtrlMSG::Stop => self.stop(),
                    CtrlMSG::PlayPause(p) => self.playpause(p),
                    CtrlMSG::Tick => self.tick(),
                    // Dev
                    CtrlMSG::In(input) => self.cpu.input_handler(input),
                    // Loader
                    CtrlMSG::LoadProg(fname) => self.loadprog(fname),
                    // Settings
                    CtrlMSG::SetRate(rate) => self.tick_rate = rate,
                    CtrlMSG::SetTurbo(t) => self.turbo = t,
                    CtrlMSG::SetMemSize(size) => self.cpu.debug_memresize(size),
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
        self.cpu.debug_clear_cu();
        self.cpu.running = true;
        self.cpu.debug_set_halt(false);
        self.t_last_update = None;
    }

    fn stop(&mut self) {
        self.t_last_update = None;
        self.cpu.running = false;
    }

    fn playpause(&mut self, p: bool) {
        self.t_last_update = None;
        self.playing = p;
    }

    fn loadprog(&mut self, prog: String) {
        self.cpu.running = false;
        self.loaded_prog = prog;
        loader::load_program(&mut self.cpu, &self.loaded_prog);
    }

    fn tick(&mut self) {
        if self.cpu.waiting_for_io
            || self.cpu.debug_get_halt()
            || !self.cpu.running
            || false
            || false
        {
            return;
        }
        if let Some(t) = self.t_last_cpu_tick {
            self.perfmon.add_sample(Instant::now() - t);
        }
        self.t_last_cpu_tick = Some(Instant::now());
        self.cpu.tick();
    }
}
