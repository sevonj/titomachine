/*
 * Functions to provide data from CPU/MEM
 * Used by emulator to adjust settings and
 * emu debug to pass values to gui.
 */

use super::{CPU, GPR};

impl CPU {
    pub fn debug_get_gprs(&mut self) -> [i32; 8] {
        self.gpr
    }

    #[allow(dead_code)] // used by tests
    pub fn debug_get_gpr(&mut self, idx: usize) -> i32 {
        self.gpr[idx]
    }
    pub fn debug_get_cu(&mut self) -> [i32; 4] {
        [self.cu_pc, self.cu_ir, self.cu_tr, self.cu_sr]
    }
    pub fn debug_get_cu_pc(&self) -> i32 { self.cu_pc }
    pub fn debug_set_cu_pc(&mut self, value: i32) {
        self.cu_pc = value;
    }
    pub fn debug_get_mmu(&mut self) -> [i32; 4] {
        [
            self.mmu_base as i32,
            self.mmu_limit as i32,
            self.mmu_mar as i32,
            self.mmu_mbr,
        ]
    }
    pub fn debug_set_gpr(&mut self, idx: GPR, value: i32) {
        self.gpr[idx as usize] = value;
    }
    #[allow(dead_code)] // used by tests
    pub fn debug_get_ivt(&mut self, idx: usize) -> i32 {
        self.ivt[idx]
    }
    pub fn debug_set_ivt(&mut self, idx: usize, value: i32) {
        self.ivt[idx] = value
    }
}
