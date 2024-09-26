// SPDX-FileCopyrightText: 2024 sevonj
// SPDX-License-Identifier: MPL-2.0

//! This is what you interact with

use crate::b91::B91;
use crate::machine::cpu::CPU;
use crate::machine::devices::{Bus, Device};

pub mod cpu;
pub mod devices;

pub struct Machine {
    cpu: CPU,
    bus: Bus,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            bus: Bus::new(),
        }
    }

    /// Advance the state by one CPU instruction.
    pub fn tick(&mut self) {
        // dev update
        if self.cpu.halt {
            return;
        }
        self.cpu.tick(&mut self.bus);
    }

    /// Fully reset the machine.
    pub fn reset(&mut self) {
        self.bus.reset();
        self.cpu.init();
        self.bus.ram.reset();
        self.bus.display.reset();
    }

    pub fn reset_soft(&mut self) {
        self.cpu.init();
        self.bus.display.reset();
    }

    /// Load a program to memory and init CPU.
    pub fn load_b91(&mut self, b91: B91) {
        // Load code segment
        let mut mem_off = b91.code_segment.start;
        for instruction in &b91.code_segment.content {
            let _ = self.bus.write(mem_off as u32, *instruction);
            mem_off += 1;
        }

        // Load data segment
        let mut mem_off = b91.data_segment.start;
        for variable in &b91.data_segment.content {
            let _ = self.bus.write(mem_off as u32, *variable);
            mem_off += 1;
        }

        // CPU init
        self.reset_soft();
        self.cpu.cu_pc = b91.code_segment.start as i32;
        self.cpu.gpr[6] = b91.code_segment.end as i32;
        self.cpu.gpr[7] = b91.data_segment.end as i32;
    }

    /// Read a memory address
    pub fn debug_read_mem(&mut self, addr: u32) -> Result<i32, ()> {
        self.bus.read(addr)
    }

    /// Write to a memory address
    pub fn debug_write_mem(&mut self, addr: u32, value: i32) -> Result<(), ()> {
        self.bus.write(addr, value)
    }

    /// Get all general purpose registers R0..=R7
    pub fn debug_get_gprs(&mut self) -> [i32; 8] {
        self.cpu.gpr
    }

    /// Get a general purpose register
    pub fn debug_get_gpr(&mut self, idx: usize) -> Result<i32, ()> {
        if idx <= self.cpu.gpr.len() {
            return Err(());
        }
        Ok(self.cpu.gpr[idx])
    }

    /// Set the value of a general purpose register
    pub fn debug_set_gpr(&mut self, idx: usize, value: i32) -> Result<(), ()> {
        if idx <= self.cpu.gpr.len() {
            return Err(());
        }
        self.cpu.gpr[idx] = value;
        Ok(())
    }

    pub fn debug_get_cpu_sp(&mut self) -> i32 {
        self.cpu.gpr[6]
    }

    pub fn debug_set_cpu_sp(&mut self, value: i32) {
        self.cpu.gpr[6] = value
    }

    pub fn debug_get_cpu_fp(&mut self) -> i32 {
        self.cpu.gpr[7]
    }

    pub fn debug_set_cpu_fp(&mut self, value: i32) {
        self.cpu.gpr[7] = value
    }

    pub fn debug_get_cpu_pc(&mut self) -> i32 {
        self.cpu.cu_pc
    }

    pub fn debug_set_cpu_pc(&mut self, value: i32) {
        self.cpu.cu_pc = value
    }

    /// Get CPU halt
    pub fn debug_get_halt(&mut self) -> bool {
        self.cpu.halt
    }

    /// Set CPU halt. Returns error if the CPU has HCF'd.
    pub fn debug_set_halt(&mut self, halt: bool) -> Result<(), ()> {
        if self.cpu.burn {
            return Err(());
        }
        self.cpu.halt = halt;
        Ok(())
    }

    /// Get CPU Halt & Catch fire status
    pub fn debug_get_hcf(&mut self) -> bool {
        self.cpu.burn
    }
}
