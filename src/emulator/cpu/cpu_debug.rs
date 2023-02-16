/*
 * Functions to provide data from CPU/MEM
 * Used by emulator to adjust settings and
 * emu debug to pass values to gui.
 */

use super::{CPU, FP, SP};
use std::ops::Range;

impl CPU {
    pub fn debug_get_gprs(&mut self) -> [i32; 8] {
        self.gpr
    }
    pub fn debug_get_cu(&mut self) -> [i32; 4] {
        [self.cu_pc, self.cu_ir, self.cu_tr, self.cu_sr]
    }
    pub fn debug_get_mmu(&mut self) -> [i32; 4] {
        [
            self.mmu_base as i32,
            self.mmu_limit as i32,
            self.mmu_mar as i32,
            self.mmu_mbr,
        ]
    }
    pub fn debug_setgpr(&mut self, index: usize, value: i32) {
        self.gpr[index] = value;
    }
    pub fn debug_get_sp(&mut self) -> i32 {
        self.gpr[SP]
    }
    pub fn debug_set_sp(&mut self, value: i32) {
        self.gpr[SP] = value;
    }
    pub fn debug_get_fp(&mut self) -> i32 {
        self.gpr[FP]
    }
    pub fn debug_set_fp(&mut self, value: i32) {
        self.gpr[FP] = value;
    }
    pub fn debug_clear_cu(&mut self) {
        self.cu_pc = 0;
        self.cu_ir = 0;
        self.cu_tr = 0;
        self.cu_sr = 0;
    }
    pub fn debug_clear_gprs(&mut self) {
        for i in &mut self.gpr {
            *i = 0;
        }
    }
    pub fn debug_clear_all_regs(&mut self) {
        self.cu_pc = 0;
        self.cu_ir = 0;
        self.cu_tr = 0;
        self.cu_sr = 0;
        for i in &mut self.gpr {
            *i = 0;
        }
    }
    pub fn debug_get_halt(&mut self) -> bool {
        self.halt
    }
    pub fn debug_set_halt(&mut self, halt: bool) {
        self.halt = halt
    }

    // Memory
    pub fn debug_memlen(&mut self) -> usize {
        self.memory.len()
    }
    pub fn debug_memclear(&mut self) {
        let len = self.memory.len();
        self.memory.clear();
        self.memory.resize(len, 0);
    }
    pub fn debug_memresize(&mut self, size: usize) {
        self.memory.clear();
        self.memory.resize(size, 0);
    }
    pub fn debug_memread(&mut self, addr: usize) -> i32 {
        self.memory[addr]
    }
    pub fn debug_memwrite(&mut self, addr: usize, value: i32) {
        self.memory[addr] = value;
    }
    pub fn debug_memread_range(&mut self, range: Range<usize>) -> Vec<i32> {
        let mut retvec: Vec<i32> = Vec::new();
        for addr in range {
            retvec.push(self.memory[addr])
        }
        retvec
    }
    pub fn debug_memwrite_range(&mut self, range: Range<usize>, values: Vec<i32>) {
        for (i, addr) in range.enumerate() {
            self.memory[addr] = values[i];
        }
    }
}
