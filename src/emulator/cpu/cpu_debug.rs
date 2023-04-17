/*
 * Functions to provide data from CPU/MEM
 * Used by emulator to adjust settings and
 * emu debug to pass values to gui.
 */

use super::{CPU, FP, SP};

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
    pub fn init(&mut self) {
        self.cu_pc = 0;
        self.cu_ir = 0;
        self.cu_tr = 0;
        self.cu_sr = 0;
        self.halt = false;
        self.burn = false;
    }
    pub fn debug_get_halt(&mut self) -> bool {
        self.halt
    }
    pub fn debug_get_ivt(&mut self, idx: usize) -> i32 {
        self.ivt[idx]
    }
    pub fn debug_set_ivt(&mut self, idx: usize, value: i32) {
        self.ivt[idx] = value
    }
    pub fn debug_print_regs(&mut self) {
        println!("\nCPU Status:\n");
        println!("HALT:    {}\nOn Fire: {}\n", self.halt, self.burn);
        println!(
            "Control Unit\n  PC: {}\n  IR: {}\n  TR: {}\n  SR: {:32b}\n",
            self.cu_pc, self.cu_ir, self.cu_tr, self.cu_sr
        );
        println!("Interrupt Vector Table");
        for (i, val) in self.ivt.into_iter().enumerate() {
            println!(" {:2}: {:x}", i, val);
        }
    }
}
