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
const SVC: u16 = 0x70;

impl CPU {
    pub fn exec_instruction(&mut self) {
        let opcode = (self.cu_ir >> 24) as u16;
        let rj = (self.cu_ir >> 21) & 0x7;
        let mode = (self.cu_ir >> 19) & 0x3;
        let ri = (self.cu_ir >> 16) & 0x7;
        let addr = (self.cu_ir & 0xffff) as i16 as i32;
        // these casts catch the sign

        self.cu_tr = addr;

        // Get Immediate operand
        match self.cu_tr.checked_add(self.gpr[ri as usize]) {
            Some(i) => self.cu_tr = i,
            None => self.cu_sr |= SR_O,
        }

        // Direct:   if mode == 1, get from mem once
        // Indirect: if mode == 2, get from mem twice
        for _ in 1..=mode {
            self.cu_tr = self.memread(self.cu_tr);
        }

        match opcode {
            NOP => {
                self.cu_pc += 1;
            }
            STORE => {
                self.memwrite(self.cu_tr, self.gpr[rj as usize]);
                self.cu_pc += 1;
            }
            LOAD => {
                self.gpr[rj as usize] = self.cu_tr;
                self.cu_pc += 1;
            }
            IN => {
                //TODO: proper devices
                self.input_wait = Some(self.cu_tr);
            }
            OUT => {
                let dev = self.cu_tr;
                let val = self.gpr[rj as usize];
                self.output = Some((dev, val));
                self.cu_pc += 1;
            }
            ADD => {
                match self.gpr[rj as usize].checked_add(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
                self.cu_pc += 1;
            }
            SUB => {
                match self.gpr[rj as usize].checked_sub(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
                self.cu_pc += 1;
            }
            MUL => {
                match self.gpr[rj as usize].checked_mul(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
                self.cu_pc += 1;
            }
            DIV => {
                match self.gpr[rj as usize].checked_div(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
                self.cu_pc += 1;
            }
            MOD => {
                self.gpr[rj as usize] %= self.cu_tr;
                self.cu_pc += 1;
            }
            AND => {
                self.gpr[rj as usize] &= self.cu_tr;
                self.cu_pc += 1;
            }
            OR => {
                self.gpr[rj as usize] |= self.cu_tr;
                self.cu_pc += 1;
            }
            XOR => {
                self.gpr[rj as usize] ^= self.cu_tr;
                self.cu_pc += 1;
            }
            SHL => {
                self.gpr[rj as usize] <<= self.cu_tr;
                self.cu_pc += 1;
            }
            SHR => {
                // Casting to unsigned because signed int defaults to arithmetic shift.
                // This tactic worked in C, TODO: verify that it works here.
                self.gpr[rj as usize] = (self.gpr[rj as usize] as u32 >> self.cu_tr) as i32;
                self.cu_pc += 1;
            }
            NOT => {
                self.gpr[rj as usize] = !self.cu_tr;
                self.cu_pc += 1;
            }
            SHRA => {
                self.gpr[rj as usize] >>= self.cu_tr;
                self.cu_pc += 1;
            }
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
                self.cu_pc += 1;
            }
            // Branching instructions
            JUMP => {
                self.cu_pc = self.cu_tr;
            }
            // Jumps that use GPR
            JNEG => {
                if self.gpr[rj as usize] < 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JZER => {
                if self.gpr[rj as usize] == 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JPOS => {
                if self.gpr[rj as usize] > 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JNNEG => {
                if self.gpr[rj as usize] >= 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JNZER => {
                if self.gpr[rj as usize] != 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JNPOS => {
                if self.gpr[rj as usize] <= 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            // Jumps that use SR
            JLES => {
                if self.cu_sr & SR_L == SR_L {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JEQU => {
                if self.cu_sr & SR_E == SR_E {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JGRE => {
                if self.cu_sr & SR_G == SR_G {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JNLES => {
                if self.cu_sr & SR_L == 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JNEQU => {
                if self.cu_sr & SR_E == 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            JNGRE => {
                if self.cu_sr & SR_G == 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            // Subroutine instructions
            CALL => {
                self.gpr[SP] += 1;
                self.memwrite(self.gpr[SP], self.cu_pc);
                self.gpr[SP] += 1;
                self.memwrite(self.gpr[SP], self.gpr[FP]);
                self.cu_pc = self.cu_tr;
                self.gpr[FP] = self.gpr[SP];
            }
            EXIT => {
                self.gpr[SP] = self.gpr[FP] - 2 - self.cu_tr;
                self.cu_pc = self.memread(self.gpr[FP] - 1);
                self.gpr[FP] = self.memread(self.gpr[FP]);
                self.cu_pc += 1;
            }
            // Stack instructions
            PUSH => {
                self.gpr[SP] += 1;
                self.memwrite(self.gpr[SP], self.cu_tr);
                self.cu_pc += 1;
            }
            POP => {
                self.gpr[ri as usize] = self.memread(self.gpr[SP]);
                self.gpr[SP] -= 1;
                self.cu_pc += 1;
            }
            PUSHR => {
                for i in 0..7 {
                    self.gpr[SP] += 1;
                    self.memwrite(self.gpr[SP], self.gpr[i]);
                }
                self.cu_pc += 1;
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
                    self.gpr[i as usize] = self.memread(addr);
                    self.gpr[SP] -= 1;
                }
                self.cu_pc += 1;
            }
            // Syscalls
            SVC => {
                self.cu_sr |= SR_S;
                self.cu_pc += 1;
            }
            _ => self.cu_sr |= SR_U,
        }
    }
}
