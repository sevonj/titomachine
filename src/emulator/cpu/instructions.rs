use super::{CPU, FP, SP, SR_D, SR_E, SR_G, SR_I, SR_L, SR_M, SR_O, SR_P, SR_S, SR_U, SR_Z};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive, ToPrimitive)]
pub enum Opcode {
    NOP = 0x00,
    STORE = 0x01,
    LOAD = 0x02,
    IN = 0x03,
    OUT = 0x04,
    ADD = 0x11,
    SUB = 0x12,
    MUL = 0x13,
    DIV = 0x14,
    MOD = 0x15,
    AND = 0x16,
    OR = 0x17,
    XOR = 0x18,
    SHL = 0x19,
    SHR = 0x1A,
    NOT = 0x1B,
    SHRA = 0x1C,
    COMP = 0x1F,
    JUMP = 0x20,
    JNEG = 0x21,
    JZER = 0x22,
    JPOS = 0x23,
    JNNEG = 0x24,
    JNZER = 0x25,
    JNPOS = 0x26,
    JLES = 0x27,
    JEQU = 0x28,
    JGRE = 0x29,
    JNLES = 0x2A,
    JNEQU = 0x2B,
    JNGRE = 0x2C,
    CALL = 0x31,
    EXIT = 0x32,
    PUSH = 0x33,
    POP = 0x34,
    PUSHR = 0x35,
    POPR = 0x36,
    SVC = 0x70,
}

impl CPU {
    pub fn exec_instruction(&mut self) {
        let opcode = self.cu_ir >> 24;
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

        match FromPrimitive::from_i32(opcode) {
            Some(Opcode::NOP) => {
                self.cu_pc += 1;
            }
            Some(Opcode::STORE) => {
                self.memwrite(self.cu_tr, self.gpr[rj as usize]);
                self.cu_pc += 1;
            }
            Some(Opcode::LOAD) => {
                self.gpr[rj as usize] = self.cu_tr;
                self.cu_pc += 1;
            }
            Some(Opcode::IN) => {
                //TODO: proper devices
                self.waiting_for_io = true;
            }
            Some(Opcode::OUT) => {
                //
                self.output = Some(self.gpr[rj as usize]);
                self.cu_pc += 1;
            }
            Some(Opcode::ADD) => {
                match self.gpr[rj as usize].checked_add(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
                self.cu_pc += 1;
            }
            Some(Opcode::SUB) => {
                match self.gpr[rj as usize].checked_sub(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
                self.cu_pc += 1;
            }
            Some(Opcode::MUL) => {
                match self.gpr[rj as usize].checked_mul(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
                self.cu_pc += 1;
            }
            Some(Opcode::DIV) => {
                match self.gpr[rj as usize].checked_div(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.cu_sr |= SR_O,
                }
                self.cu_pc += 1;
            }
            Some(Opcode::MOD) => {
                self.gpr[rj as usize] %= self.cu_tr;
                self.cu_pc += 1;
            }
            Some(Opcode::AND) => {
                self.gpr[rj as usize] &= self.cu_tr;
                self.cu_pc += 1;
            }
            Some(Opcode::OR) => {
                self.gpr[rj as usize] |= self.cu_tr;
                self.cu_pc += 1;
            }
            Some(Opcode::XOR) => {
                self.gpr[rj as usize] ^= self.cu_tr;
                self.cu_pc += 1;
            }
            Some(Opcode::SHL) => {
                self.gpr[rj as usize] <<= self.cu_tr;
                self.cu_pc += 1;
            }
            Some(Opcode::SHR) => {
                // Casting to unsigned because signed int defaults to arithmetic shift.
                // This tactic worked in C, TODO: verify that it works here.
                self.gpr[rj as usize] = (self.gpr[rj as usize] as u32 >> self.cu_tr) as i32;
                self.cu_pc += 1;
            }
            Some(Opcode::NOT) => {
                self.gpr[rj as usize] = !self.cu_tr;
                self.cu_pc += 1;
            }
            Some(Opcode::SHRA) => {
                self.gpr[rj as usize] >>= self.cu_tr;
                self.cu_pc += 1;
            }
            Some(Opcode::COMP) => {
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
            Some(Opcode::JUMP) => {
                self.cu_pc = self.cu_tr;
            }
            // Jumps that use GPR
            Some(Opcode::JNEG) => {
                if self.gpr[rj as usize] < 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JZER) => {
                if self.gpr[rj as usize] == 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JPOS) => {
                if self.gpr[rj as usize] > 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JNNEG) => {
                if self.gpr[rj as usize] >= 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JNZER) => {
                if self.gpr[rj as usize] != 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JNPOS) => {
                if self.gpr[rj as usize] <= 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            // Jumps that use SR
            Some(Opcode::JLES) => {
                if self.cu_sr & SR_L == SR_L {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JEQU) => {
                if self.cu_sr & SR_E == SR_E {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JGRE) => {
                if self.cu_sr & SR_G == SR_G {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JNLES) => {
                if self.cu_sr & SR_L == 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JNEQU) => {
                if self.cu_sr & SR_E == 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            Some(Opcode::JNGRE) => {
                if self.cu_sr & SR_G == 0 {
                    self.cu_pc = self.cu_tr;
                } else {
                    self.cu_pc += 1;
                }
            }
            // Subroutine instructions
            Some(Opcode::CALL) => {
                self.gpr[SP] += 1;
                self.memwrite(self.gpr[SP], self.cu_pc);
                self.gpr[SP] += 1;
                self.memwrite(self.gpr[SP], self.gpr[FP]);
                self.cu_pc = self.cu_tr;
                self.gpr[FP] = self.gpr[SP];
            }
            Some(Opcode::EXIT) => {
                self.gpr[SP] = self.gpr[FP] - 2 - self.cu_tr;
                self.cu_pc = self.memread(self.gpr[FP] - 1);
                self.gpr[FP] = self.memread(self.gpr[FP]);
                self.cu_pc += 1;
            }
            // Stack instructions
            Some(Opcode::PUSH) => {
                self.gpr[SP] += 1;
                self.memwrite(self.gpr[SP], self.cu_tr);
                self.cu_pc += 1;
            }
            Some(Opcode::POP) => {
                self.gpr[ri as usize] = self.memread(self.gpr[SP]);
                self.gpr[SP] -= 1;
                self.cu_pc += 1;
            }
            Some(Opcode::PUSHR) => {
                for i in 0..7 {
                    self.gpr[SP] += 1;
                    self.memwrite(self.gpr[SP], self.gpr[i]);
                }
                self.cu_pc += 1;
            }
            Some(Opcode::POPR) => {
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
            Some(Opcode::SVC) => {
                self.cu_sr |= SR_S;
                self.cu_pc += 1;
            }
            None => {
                self.cu_sr |= SR_U;
            }
        }
    }
}
