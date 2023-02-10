use super::{
    instance::{FP, SP, SR_E, SR_G, SR_L, SR_M, SR_O, SR_S, SR_U},
    Emu, ReplyMSG,
};

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

impl TryFrom<i32> for Opcode {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == 0 as i32 => Ok(Opcode::NOP),
            x if x == 1 as i32 => Ok(Opcode::NOP),
            x if x == 2 as i32 => Ok(Opcode::NOP),
            _ => Err(()),
        }
    }
}

pub struct TTK91Instruction {
    pub opcode: i32,
    pub rj: i32,
    pub mode: i32,
    pub ri: i32,
    pub addr: i32,
}

impl TTK91Instruction {
    pub fn from(input_instr: i32) -> Self {
        TTK91Instruction {
            opcode: (input_instr >> 24),
            rj: ((input_instr >> 21) & 0x7),
            mode: ((input_instr >> 19) & 0x3),
            ri: ((input_instr >> 16) & 0x7),
            // these casts catch the sign
            addr: (input_instr & 0xffff) as i16 as i32,
        }
    }
}

#[derive(FromPrimitive, ToPrimitive)]
enum builtin_symbols {
    dev_crt = 0,
    dev_kbd = 1,

    svc_halt = 11,
    svc_read = 12,
    svc_write = 13,
    svc_time = 14,
    svc_date = 15,
}
impl Emu {
    pub fn exec(&mut self) {
        if !self.instance.running {
            println!("Err: called exec() but the machine is not turned on! You shouldn't be here.");
            return;
        }
        if self.instance.halted {
            println!("Err: called exec() but the machine is halted! You shouldn't be here.");
            return;
        }
        if self.instance.pc as usize >= self.instance.memory.len() {
            self.instance.sr |= SR_M;
            return;
        }
        self.instance.ir = self.instance.memory[self.instance.pc as usize];
        let instruction = TTK91Instruction::from(self.instance.ir);
        self.instance.tr = instruction.addr;

        // Second register
        if instruction.ri != 0 {
            if instruction.ri as usize >= self.instance.memory.len() {
                self.instance.sr |= SR_M;
                return;
            }
            match self.instance.tr.checked_add(self.instance.gpr[instruction.ri as usize]){
                Some(i) =>  self.instance.tr = i,
                None => self.instance.sr |= SR_O,
            }
        }
        // Direct addressing: value is a pointer
        if instruction.mode == 1 {
            if self.instance.tr as usize >= self.instance.memory.len() {
                self.instance.sr |= SR_M;
                return;
            }
            self.instance.tr = self.instance.memory[self.instance.tr as usize];
        }
        // Indirect addressing: value is a pointer to a pointer
        else if instruction.mode == 2 {
            if self.instance.tr as usize >= self.instance.memory.len() {
                self.instance.sr |= SR_M;
                return;
            }
            self.instance.tr =
                self.instance.memory[self.instance.memory[self.instance.tr as usize] as usize];
        }

        match FromPrimitive::from_i32(instruction.opcode) {
            Some(Opcode::NOP) => {
                self.instance.pc += 1;
            }
            Some(Opcode::STORE) => {
                if self.instance.tr as usize >= self.instance.memory.len() {
                    self.instance.sr |= SR_M;
                    return;
                }
                self.instance.memory[self.instance.tr as usize] =
                    self.instance.gpr[instruction.rj as usize];
                self.instance.pc += 1;
            }
            Some(Opcode::LOAD) => {
                if self.instance.tr as usize >= self.instance.memory.len() {
                    self.instance.sr |= SR_M;
                    return;
                }
                self.instance.gpr[instruction.rj as usize] = self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::IN) => {
                self.tx.send(ReplyMSG::In);
                self.instance.waiting_for_io = true;
            }
            Some(Opcode::OUT) => {
                self.tx
                    .send(ReplyMSG::Out(self.instance.gpr[instruction.rj as usize]));
                self.instance.pc += 1;
            }
            Some(Opcode::ADD) => {
                match self.instance.gpr[instruction.rj as usize].checked_add(self.instance.tr) {
                    Some(i) => self.instance.gpr[instruction.rj as usize] = i,
                    None => self.instance.sr |= SR_O,
                }
                self.instance.pc += 1;
            }
            Some(Opcode::SUB) => {
                match self.instance.gpr[instruction.rj as usize].checked_sub(self.instance.tr) {
                    Some(i) => self.instance.gpr[instruction.rj as usize] = i,
                    None => self.instance.sr |= SR_O,
                }
                self.instance.pc += 1;
            }
            Some(Opcode::MUL) => {
                match self.instance.gpr[instruction.rj as usize].checked_mul(self.instance.tr) {
                    Some(i) => self.instance.gpr[instruction.rj as usize] = i,
                    None => self.instance.sr |= SR_O,
                }
                self.instance.pc += 1;
            }
            Some(Opcode::DIV) => {
                match self.instance.gpr[instruction.rj as usize].checked_div(self.instance.tr) {
                    Some(i) => self.instance.gpr[instruction.rj as usize] = i,
                    None => self.instance.sr |= SR_O,
                }
                self.instance.pc += 1;
            }
            Some(Opcode::MOD) => {
                self.instance.gpr[instruction.rj as usize] %= self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::AND) => {
                self.instance.gpr[instruction.rj as usize] &= self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::OR) => {
                self.instance.gpr[instruction.rj as usize] |= self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::XOR) => {
                self.instance.gpr[instruction.rj as usize] ^= self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::SHL) => {
                self.instance.gpr[instruction.rj as usize] <<= self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::SHR) => {
                // Casting to unsigned because signed int defaults to arithmetic shift.
                // This tactic worked in C, TODO: verify that it works here.
                self.instance.gpr[instruction.rj as usize] =
                    (self.instance.gpr[instruction.rj as usize] as u32 >> self.instance.tr) as i32;
                self.instance.pc += 1;
            }
            Some(Opcode::NOT) => {
                self.instance.gpr[instruction.rj as usize] = !self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::SHRA) => {
                self.instance.gpr[instruction.rj as usize] >>= self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::COMP) => {
                if self.instance.gpr[instruction.rj as usize] > self.instance.tr {
                    // Greater
                    self.instance.sr |= SR_G;
                    self.instance.sr &= !SR_E;
                    self.instance.sr &= !SR_L;
                } else if self.instance.gpr[instruction.rj as usize] == self.instance.tr {
                    // Equal
                    self.instance.sr &= !SR_G;
                    self.instance.sr |= SR_E;
                    self.instance.sr &= !SR_L;
                } else {
                    // Less
                    self.instance.sr &= !SR_G;
                    self.instance.sr &= !SR_E;
                    self.instance.sr |= SR_L;
                }
                self.instance.pc += 1;
            }
            // Branching instructions
            Some(Opcode::JUMP) => {
                self.instance.pc = self.instance.tr;
            }
            // Jumps that use GPR
            Some(Opcode::JNEG) => {
                if self.instance.gpr[instruction.rj as usize] < 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JZER) => {
                if self.instance.gpr[instruction.rj as usize] == 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JPOS) => {
                if self.instance.gpr[instruction.rj as usize] > 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JNNEG) => {
                if self.instance.gpr[instruction.rj as usize] >= 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JNZER) => {
                if self.instance.gpr[instruction.rj as usize] != 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JNPOS) => {
                if self.instance.gpr[instruction.rj as usize] <= 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            // Jumps that use SR
            Some(Opcode::JLES) => {
                if self.instance.sr & SR_L == SR_L {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JEQU) => {
                if self.instance.sr & SR_E == SR_E {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JGRE) => {
                if self.instance.sr & SR_G == SR_G {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JNLES) => {
                if self.instance.sr & SR_L == 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JNEQU) => {
                if self.instance.sr & SR_E == 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            Some(Opcode::JNGRE) => {
                if self.instance.sr & SR_G == 0 {
                    self.instance.pc = self.instance.tr;
                } else {
                    self.instance.pc += 1;
                }
            }
            // Subroutine instructions
            Some(Opcode::CALL) => {
                self.instance.gpr[SP] += 1;
                self.instance.memory[self.instance.gpr[SP] as usize] = self.instance.pc;
                self.instance.gpr[SP] += 1;
                self.instance.memory[self.instance.gpr[SP] as usize] = self.instance.gpr[FP];
                self.instance.pc = self.instance.tr;
                self.instance.gpr[FP] = self.instance.gpr[SP];
            }
            Some(Opcode::EXIT) => {
                self.instance.gpr[SP] = self.instance.gpr[FP] - 2 - self.instance.tr;
                self.instance.pc = self.instance.memory[self.instance.gpr[FP] as usize - 1];
                self.instance.gpr[FP] = self.instance.memory[self.instance.gpr[FP] as usize];
                self.instance.pc += 1;
            }
            // Stack instructions
            Some(Opcode::PUSH) => {
                self.instance.gpr[SP] += 1;
                self.instance.memory[self.instance.gpr[SP] as usize] = self.instance.tr;
                self.instance.pc += 1;
            }
            Some(Opcode::POP) => {
                self.instance.gpr[instruction.ri as usize] =
                    self.instance.memory[self.instance.gpr[SP] as usize];
                self.instance.gpr[SP] -= 1;
                self.instance.pc += 1;
            }
            Some(Opcode::PUSHR) => {
                for i in 0..7 {
                    self.instance.gpr[SP] += 1;
                    self.instance.memory[self.instance.gpr[SP] as usize] = self.instance.gpr[i];
                }
                self.instance.pc += 1;
            }
            Some(Opcode::POPR) => {
                let old_sp = self.instance.gpr[SP] as usize;
                for i in (0..7).rev() {
                    self.instance.gpr[i] = self.instance.memory[(old_sp - 6 + i)];
                    self.instance.gpr[SP] -= 1;
                }
                self.instance.pc += 1;
            }
            // Syscalls
            Some(Opcode::SVC) => {
                self.instance.sr |= SR_S;
                self.instance.pc += 1;
            }
            None => {
                self.instance.sr |= SR_U;
            }
        }
    }

    pub fn svc_handler(&mut self) {
        if self.instance.sr & SR_S == 0 {
            return;
        }
        self.instance.sr &= !SR_S; // Clear syscall flag

        match FromPrimitive::from_i32(self.instance.tr) {
            Some(builtin_symbols::svc_halt) => {
                println!("SVC: System halted.");
                self.instance.halted = true;
            }
            Some(builtin_symbols::svc_read) => {
                println!("SVC: ERR: READ not implemented!");
                self.instance.halted = true;
            }
            Some(builtin_symbols::svc_write) => {
                println!("SVC: ERR: WRITE not implemented!");
                self.instance.halted = true;
            }
            Some(builtin_symbols::svc_time) => {
                println!("SVC: ERR: TIME not implemented!");
                self.instance.halted = true;
            }
            Some(builtin_symbols::svc_date) => {
                println!("SVC: ERR: DATE not implemented!");
                self.instance.halted = true;
            }
            _ => {
                println!("SVC: ERR: Unknown request!");
                self.instance.halted = true;
            }
        }
    }
    pub fn input_handler(&mut self, input: i32) {
        if self.instance.waiting_for_io == false {
            panic!("input_handler(): waiting_for_io is false. Why did you call me?")
        }
        let instruction = TTK91Instruction::from(self.instance.ir);
        self.instance.gpr[instruction.rj as usize] = input;
        self.instance.waiting_for_io = false;
        self.instance.pc += 1;
    }
}
