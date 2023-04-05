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
    LoadProg(String),
    ClearMem,
    SetRate(f32),
    SetTurbo(bool),
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
    Display(Vec<image::Rgba<u8>>),
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
        let speed_percent = (1. / 120.) / self.perfmon.get_last_duration() * 100.;
        match self.tx.send(ReplyMSG::State(EmuState {
            playing: self.playing,
            running: self.running,
            halted: self.cpu.debug_get_halt(),
            speed_percent,
        })) {
            Ok(_) => (),
            Err(_) => todo!(),
        }
    }

    pub fn debug_sendmem(&mut self, range: Range<i32>) {
        let mut retvec: Vec<i32> = Vec::with_capacity(range.len());
        for i in range.clone() {
            if let Ok(val) = self.bus.read(i) {
                retvec.push(val);
            } else {
                break;
            };
        }
        match self.tx.send(ReplyMSG::Mem(retvec)) {
            Ok(_) => (),
            Err(_) => todo!(),
        }
        match self.tx.send(ReplyMSG::MemSize(0x2000)) {
            Ok(_) => (),
            Err(_) => todo!(),
        }
    }

    pub fn debug_sendregs(&mut self) {
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

    pub fn debug_senddisp(&mut self) {
        //let range = 8192..8192 + 120 * 160;
        //let retvec = self.cpu.debug_memread_range(range).to_vec();
        match self
            .tx
            .send(ReplyMSG::Display(self.bus.display.debug_get_framebuf()))
        {
            Ok(_) => (),
            Err(_) => todo!(),
        }
    }
}
