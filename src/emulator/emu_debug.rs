/*
 * Functions that send data from Emu to GUI thread
 *
 */

use super::Emu;
use std::ops::Range;
use libttktk::b91::B91;

pub enum CtrlMSG {
    PlaybackStart,
    PlaybackStop,
    PlaybackPlayPause(bool),
    PlaybackTick,
    LoadB91(B91),
    Reset(),
    ClearMem,
    SetRate(f32),
    SetTurbo(bool),
    GetState,
    GetMem(Range<u32>),
    EnableBreakpoints(bool),
    ClearBreakpoints,
    InsertBreakpoint(usize),
    RemoveBreakpoint(usize),
}

pub enum ReplyMSG {
    State(EmuState),
    Regs(DebugRegs),
    Mem(Vec<i32>),
    SegmentOffsets(usize, usize, usize),
}

pub struct EmuState {
    pub playing: bool,
    pub running: bool,
    pub halted: bool,
    pub speed_percent: f32,
}

pub struct DebugRegs {
    pub pc: i32,
    pub ir: i32,
    pub tr: i32,
    pub sr: i32,
    pub gpr: [i32; 8],
    pub base: i32,
    pub limit: i32,
    pub mar: i32,
    pub mbr: i32,
}

impl Default for DebugRegs {
    fn default() -> Self {
        DebugRegs {
            pc: 0,
            ir: 0,
            tr: 0,
            sr: 0,
            gpr: [0; 8],
            base: 0,
            limit: 0,
            mar: 0,
            mbr: 0,
        }
    }
}

impl Emu {
    /// Loop through any queued control messages.
    pub(crate) fn check_mail(&mut self) {
        loop {
            if let Ok(msg) = self.rx.try_recv() {
                match msg {
                    // Playback control
                    CtrlMSG::PlaybackStart => self.start(),
                    CtrlMSG::PlaybackStop => self.stop(),
                    CtrlMSG::PlaybackPlayPause(p) => self.playpause(p),
                    CtrlMSG::PlaybackTick => self.manual_tick(),
                    // Loader
                    CtrlMSG::Reset() => self.reset(),
                    CtrlMSG::LoadB91(b91) => self.load_b91(b91),
                    CtrlMSG::ClearMem => self.clearmem(),
                    // Settings
                    CtrlMSG::SetRate(rate) => self.tick_rate = rate,
                    CtrlMSG::SetTurbo(t) => self.turbo = t,
                    // Debug
                    CtrlMSG::GetState => self.debug_sendstate(),
                    CtrlMSG::GetMem(range) => self.debug_sendmem(range),
                    CtrlMSG::EnableBreakpoints(enable) => self.breakpoints_enabled = enable,
                    CtrlMSG::ClearBreakpoints => self.breakpoints.clear(),
                    CtrlMSG::InsertBreakpoint(addr) => { self.breakpoints.insert(addr); }
                    CtrlMSG::RemoveBreakpoint(addr) => { self.breakpoints.remove(&addr); }
                }
            } else {
                break;
            }
        }
    }

    pub fn debug_sendstate(&mut self) {
        let speed_percent = (1. / 60.) / self.perfmon.get_last_duration() * 100.;
        match self.tx.send(ReplyMSG::State(EmuState {
            playing: self.playing,
            running: self.running,
            halted: self.cpu.halt,
            speed_percent,
        })) {
            Ok(_) => (),
            Err(_) => todo!(),
        }
        self.debug_sendregs()
    }

    pub fn debug_sendmem(&mut self, range: Range<u32>) {
        let mut retvec: Vec<i32> = Vec::with_capacity(range.len());
        for i in range.clone() {
            if let Ok(val) = self.bus.read(i) {
                retvec.push(val);
            } else {
                break;
            };
        }
        let _ = self.tx.send(ReplyMSG::Mem(retvec));
    }

    fn debug_sendregs(&mut self) {
        let cu = self.cpu.debug_get_cu();
        let mmu = self.cpu.debug_get_mmu();
        match self.tx.send(ReplyMSG::Regs(DebugRegs {
            pc: cu[0],
            ir: cu[1],
            tr: cu[2],
            sr: cu[3],
            gpr: self.cpu.debug_get_gprs(),
            base: mmu[0],
            limit: mmu[1],
            mar: mmu[2],
            mbr: mmu[3],
        })) {
            Ok(_) => (),
            Err(_) => todo!(),
        }
    }
}
