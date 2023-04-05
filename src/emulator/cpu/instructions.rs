use crate::emulator::Bus;

use super::{CPU, FP, SP, SR_D, SR_E, SR_G, SR_I, SR_L, SR_M, SR_O, SR_P, SR_S, SR_U, SR_Z};

const NOP: u16 = 0x00;
const STORE: u16 = 0x01;
const LOAD: u16 = 0x02;
const IN: u16 = 0x03;
const OUT: u16 = 0x04;
const ADD: u16 = 0x11;
const SUB: u16 = 0x12;
const MUL: u16 = 0x13;
const DIV: u16 = 0x14;
const MOD: u16 = 0x15;
const AND: u16 = 0x16;
const OR: u16 = 0x17;
const XOR: u16 = 0x18;
const SHL: u16 = 0x19;
const SHR: u16 = 0x1A;
const NOT: u16 = 0x1B;
const SHRA: u16 = 0x1C;
const COMP: u16 = 0x1F;
const JUMP: u16 = 0x20;
const JNEG: u16 = 0x21;
const JZER: u16 = 0x22;
const JPOS: u16 = 0x23;
const JNNEG: u16 = 0x24;
const JNZER: u16 = 0x25;
const JNPOS: u16 = 0x26;
const JLES: u16 = 0x27;
const JEQU: u16 = 0x28;
const JGRE: u16 = 0x29;
const JNLES: u16 = 0x2A;
const JNEQU: u16 = 0x2B;
const JNGRE: u16 = 0x2C;
const CALL: u16 = 0x31;
const EXIT: u16 = 0x32;
const PUSH: u16 = 0x33;
const POP: u16 = 0x34;
const PUSHR: u16 = 0x35;
const POPR: u16 = 0x36;
const IEXIT: u16 = 0x39;
const SVC: u16 = 0x70;
const HLT: u16 = 0x71;
const HCF: u16 = 0x72;

impl CPU {
    pub fn exec_instruction(&mut self, bus: &mut Bus) {
        let opcode = (self.cu_ir >> 24) as u16;
        let rj = (self.cu_ir >> 21) & 0x7;
        let mode = (self.cu_ir >> 19) & 0x3;
        let ri = (self.cu_ir >> 16) & 0x7;
        let addr = (self.cu_ir & 0xffff) as i16 as i32;
        // these casts catch the sign

        self.fetch_second_operand(addr, ri, mode, bus);

        match opcode {
            NOP => {}
            STORE => self.memwrite(bus, self.cu_tr, self.gpr[rj as usize]),
            LOAD => self.gpr[rj as usize] = self.cu_tr,
            IN => match bus.read_port(self.cu_tr) {
                Ok(val) => self.gpr[rj as usize] = val,
                Err(_) => println!("input failed!"),
            },
            OUT => match bus.write_port(self.cu_tr, self.gpr[rj as usize]) {
                Ok(_) => (),
                Err(_) => (),
            },
            ADD => match self.gpr[rj as usize].checked_add(self.cu_tr) {
                Some(i) => self.gpr[rj as usize] = i,
                None => self.cu_sr |= SR_O,
            },
            SUB => match self.gpr[rj as usize].checked_sub(self.cu_tr) {
                Some(i) => self.gpr[rj as usize] = i,
                None => self.cu_sr |= SR_O,
            },
            MUL => match self.gpr[rj as usize].checked_mul(self.cu_tr) {
                Some(i) => self.gpr[rj as usize] = i,
                None => self.cu_sr |= SR_O,
            },
            DIV => {
                if self.cu_tr == 0 {
                    self.cu_sr |= SR_Z;
                    return;
                }
                match self.gpr[rj as usize].checked_div(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
            }
            MOD => self.gpr[rj as usize] %= self.cu_tr,
            AND => self.gpr[rj as usize] &= self.cu_tr,
            OR => self.gpr[rj as usize] |= self.cu_tr,
            XOR => self.gpr[rj as usize] ^= self.cu_tr,
            SHL => self.gpr[rj as usize] <<= self.cu_tr,
            SHR => {
                // Casting to unsigned because signed int defaults to arithmetic shift.
                // This tactic worked in C, TODO: verify that it works here.
                self.gpr[rj as usize] = (self.gpr[rj as usize] as u32 >> self.cu_tr) as i32;
            }
            NOT => self.gpr[rj as usize] = !self.cu_tr,
            SHRA => self.gpr[rj as usize] >>= self.cu_tr,
            COMP => {
                if self.gpr[rj as usize] > self.cu_tr {
                    // Greater
                    self.cu_sr |= SR_G;
                    self.cu_sr &= !SR_E;
                    self.cu_sr &= !SR_L;
                } else if self.gpr[rj as usize] == self.cu_tr {
                    // Equal
                    self.cu_sr &= !SR_G;
                    self.cu_sr |= SR_E;
                    self.cu_sr &= !SR_L;
                } else {
                    // Less
                    self.cu_sr &= !SR_G;
                    self.cu_sr &= !SR_E;
                    self.cu_sr |= SR_L;
                }
            }
            // Branching instructions
            JUMP => {
                self.cu_pc = self.cu_tr;
            }
            // Jumps that use GPR
            JNEG => {
                if self.gpr[rj as usize] < 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JZER => {
                if self.gpr[rj as usize] == 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JPOS => {
                if self.gpr[rj as usize] > 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNNEG => {
                if self.gpr[rj as usize] >= 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNZER => {
                if self.gpr[rj as usize] != 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNPOS => {
                if self.gpr[rj as usize] <= 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            // Jumps that use SR
            JLES => {
                if self.cu_sr & SR_L == SR_L {
                    self.cu_pc = self.cu_tr;
                }
            }
            JEQU => {
                if self.cu_sr & SR_E == SR_E {
                    self.cu_pc = self.cu_tr;
                }
            }
            JGRE => {
                if self.cu_sr & SR_G == SR_G {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNLES => {
                if self.cu_sr & SR_L == 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNEQU => {
                if self.cu_sr & SR_E == 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNGRE => {
                if self.cu_sr & SR_G == 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            // Subroutine instructions
            CALL => {
                self.gpr[SP] += 1;
                self.memwrite(bus, self.gpr[SP], self.cu_pc);
                self.gpr[SP] += 1;
                self.memwrite(bus, self.gpr[SP], self.gpr[FP]);
                self.cu_pc = self.cu_tr;
                self.gpr[FP] = self.gpr[SP];
            }
            EXIT => {
                self.gpr[SP] = self.gpr[FP] - 2 - self.cu_tr;
                self.cu_pc = self.memread(bus, self.gpr[FP] - 1);
                self.gpr[FP] = self.memread(bus, self.gpr[FP]);
            }
            // Stack instructions
            PUSH => {
                self.gpr[SP] += 1;
                self.memwrite(bus, self.gpr[SP], self.cu_tr);
            }
            POP => {
                self.gpr[ri as usize] = self.memread(bus, self.gpr[SP]);
                self.gpr[SP] -= 1;
            }
            PUSHR => {
                for i in 0..7 {
                    self.gpr[SP] += 1;
                    self.memwrite(bus, self.gpr[SP], self.gpr[i]);
                }
            }
            POPR => {
                let old_sp = self.gpr[SP];
                for i in (0..7).rev() {
                    let addr;
                    match old_sp.checked_sub(6) {
                        Some(n) => match n.checked_add(i) {
                            Some(n) => addr = n,
                            None => todo!(),
                        },
                        None => todo!(),
                    }
                    self.gpr[i as usize] = self.memread(bus, addr);
                    self.gpr[SP] -= 1;
                }
            }
            IEXIT => {
                // Pop FP, PC, SR
                self.gpr[FP] = self.memread(bus, self.gpr[SP]);
                self.cu_pc = self.memread(bus, self.gpr[SP] - 1);
                self.cu_sr = self.memread(bus, self.gpr[SP] - 2);
                self.gpr[SP] -= 3;
                // Pop params
                self.gpr[SP] -= self.cu_tr;
            }
            // Syscalls
            SVC => self.cu_sr |= SR_S,
            HLT => self.halt = true,
            HCF => {
                self.halt = true;
                self.burn = true;
                println!("Execution has ended.");
                self.debug_print_regs();
            }
            _ => self.cu_sr |= SR_U,
        }
    }

    fn fetch_second_operand(&mut self, addr: i32, ri: i32, mode: i32, bus: &mut Bus) {
        self.cu_tr = addr;

        // Get Immediate operand
        match self.cu_tr.checked_add(self.gpr[ri as usize]) {
            Some(i) => self.cu_tr = i,
            None => self.cu_sr |= SR_O,
        }

        // Direct:   if mode == 1, get from mem once
        // Indirect: if mode == 2, get from mem twice
        for _ in 1..=mode {
            self.cu_tr = self.memread(bus, self.cu_tr);
        }
    }
}
