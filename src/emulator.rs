use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
pub mod emu_debug;
mod perfmon;
use chrono::Local;

use self::cpu::{CPU, SR_I, SR_M, SR_O, SR_S, SR_U, SR_Z};
use self::emu_debug::{CtrlMSG, ReplyMSG};
use self::perfmon::PerfMonitor;
mod cpu;
mod loader;

const DEV_CRT: i32 = 0;
const DEV_KBD: i32 = 1;
const DEV_RTC: i32 = 2;
const DEV_STDIN: i32 = 6;
const DEV_STDOUT: i32 = 7;

pub fn run(tx: Sender<ReplyMSG>, rx: Receiver<CtrlMSG>) {
    let mut emu = Emu::default(tx, rx);
    loop {
        emu.update();
    }
}

pub struct Emu {
    cpu: CPU,
    tx: Sender<ReplyMSG>,
    rx: Receiver<CtrlMSG>,
    loaded_prog: String,
    running: bool,
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
            tx,
            rx,
            loaded_prog: String::new(),
            running: false,
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

    pub fn update(&mut self) {
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
            if let Ok(msg) = self.rx.try_recv() {
                match msg {
                    // Playback control
                    CtrlMSG::PlaybackStart => self.start(),
                    CtrlMSG::PlaybackStop => self.stop(),
                    CtrlMSG::PlaybackPlayPause(p) => self.playpause(p),
                    CtrlMSG::PlaybackTick => self.tick(),
                    // Dev
                    CtrlMSG::DevKbdIn(input) => self.cpu.input_handler(input),
                    CtrlMSG::DevGamepadState(_input) => todo!(),
                    // Loader
                    CtrlMSG::LoadProg(fname) => self.loadprog(fname),
                    CtrlMSG::ClearMem => self.clearmem(),
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
        loader::load_program(&mut self.cpu, &self.loaded_prog);
    }
    fn reload(&mut self) {
        loader::load_program(&mut self.cpu, &self.loaded_prog);
    }

    fn clearmem(&mut self) {
        self.stop();
        self.cpu.debug_memclear();
    }

    fn tick(&mut self) {
        if self.cpu.input_wait != None || self.cpu.debug_get_halt() || !self.running {
            return;
        }
        self.perfmon.tick();
        self.t_last_cpu_tick = Some(Instant::now());
        self.cpu.tick();

        if let Some(dev) = self.cpu.input_wait {
            self.dev_read(dev)
        }
        if let Some((dev, val)) = self.cpu.output {
            self.dev_write(dev, val);
            self.cpu.output = None;
        }
        if self.cpu.debug_is_on_fire() {
            self.playing = false;
            self.running = false;
        }
    }

    fn dev_read(&mut self, dev: i32) {
        match dev {
            DEV_CRT => {
                println!("You can't read from crt!");
                self.cpu.input_handler(0);
            }
            DEV_KBD => {
                self.tx.send(ReplyMSG::In);
            }
            DEV_RTC => {
                let time =
                    Local::now().timestamp() as i32 + Local::now().offset().local_minus_utc();
                self.cpu.input_handler(time);
            }
            _ => {
                println!("Attempted to read from an unknown device");
                self.cpu.input_handler(0);
            }
        }
    }
    fn dev_write(&mut self, dev: i32, val: i32) {
        match dev {
            DEV_CRT => {
                self.tx.send(ReplyMSG::Out(val));
            }
            DEV_KBD => {
                println!("You can't output to a keyboard!");
            }
            DEV_RTC => {
                println!("You can't output to RTC!");
            }
            _ => {
                println!("Attempted to write into an unknown device");
            }
        }
    }
}
