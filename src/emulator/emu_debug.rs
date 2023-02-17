/*
 * Functions that send data from Emu to GUI thread
 *
 */

use super::Emu;
use std::ops::Range;
pub enum CtrlMSG {
    PlaybackStart,
    PlaybackStop,
    PlaybackPlayPause(bool),
    PlaybackTick,
    DevKbdIn(i32),
    DevGamepadState(i32),
    LoadProg(String),
    SetRate(f32),
    SetTurbo(bool),
    SetMemSize(usize),
    GetState,
    GetRegs,
    GetMem(Range<i32>),
    GetDisp,
}
pub enum ReplyMSG {
    State(EmuState),
    Regs(DebugRegs),
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
    pub fn debug_sendstate(&mut self) {
        self.tx.send(ReplyMSG::State(EmuState {
            playing: self.playing,
            running: self.running,
            halted: self.cpu.debug_get_halt(),
            speed_percent: self.perfmon.get_percent(),
        }));
    }

    pub fn debug_sendmem(&mut self, range: Range<i32>) {
        let mut retvec: Vec<i32> = Vec::with_capacity(range.len());
        for i in range.clone() {
            if i >= self.cpu.debug_memlen() as i32 {
                break;
            }
            retvec.push(self.cpu.debug_memread(i as usize));
        }
        self.tx.send(ReplyMSG::Mem(retvec));
        self.tx
            .send(ReplyMSG::MemSize(self.cpu.debug_memlen()));
    }

    pub fn debug_sendregs(&mut self) {
        let cu = self.cpu.debug_get_cu();
        let mmu = self.cpu.debug_get_mmu();
        self.tx.send(ReplyMSG::Regs(DebugRegs {
            pc: cu[0],
            ir: cu[1],
            tr: cu[2],
            sr: cu[3],
            gpr: self.cpu.debug_get_gprs(),
            base: mmu[0],
            limit: mmu[1],
            mar: mmu[2],
            mbr: mmu[3],
        }));
    }

    pub fn debug_senddisp(&mut self) {
        let range = 8192..8192 + 120 * 160;
        let retvec = self.cpu.debug_memread_range(range).to_vec();
        self.tx.send(ReplyMSG::Display(retvec));
    }
}
