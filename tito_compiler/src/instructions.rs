// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! TTKTK - TTK-91 ToolKit
//!
//! TTK-91 Instructon module. Hosts TTK-91 Instruction struct, and relevant enums.
//!
use std::fmt;
use std::str::FromStr;

pub struct TTK91Instruction {
    pub opcode: OpCode,
    pub rj: Register,
    pub mode: AddressingMode,
    pub ri: Register,
    pub addr: i16,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Register {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
}

#[derive(Copy, Clone)]
pub enum AddressingMode {
    Immediate = 0,
    Direct = 1,
    Indirect = 2,
    Invalid = 3,
}

#[derive(Copy, Clone)]
pub enum OpCode {
    // Standard
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

    // Extended
    IEXIT = 0x39,
    HLT = 0x71,
    HCF = 0x72,
}

impl TryFrom<i32> for OpCode {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(OpCode::NOP),
            0x01 => Ok(OpCode::STORE),
            0x02 => Ok(OpCode::LOAD),
            0x03 => Ok(OpCode::IN),
            0x04 => Ok(OpCode::OUT),
            0x11 => Ok(OpCode::ADD),
            0x12 => Ok(OpCode::SUB),
            0x13 => Ok(OpCode::MUL),
            0x14 => Ok(OpCode::DIV),
            0x15 => Ok(OpCode::MOD),
            0x16 => Ok(OpCode::AND),
            0x17 => Ok(OpCode::OR),
            0x18 => Ok(OpCode::XOR),
            0x19 => Ok(OpCode::SHL),
            0x1A => Ok(OpCode::SHR),
            0x1B => Ok(OpCode::NOT),
            0x1C => Ok(OpCode::SHRA),
            0x1F => Ok(OpCode::COMP),
            0x20 => Ok(OpCode::JUMP),
            0x21 => Ok(OpCode::JNEG),
            0x22 => Ok(OpCode::JZER),
            0x23 => Ok(OpCode::JPOS),
            0x24 => Ok(OpCode::JNNEG),
            0x25 => Ok(OpCode::JNZER),
            0x26 => Ok(OpCode::JNPOS),
            0x27 => Ok(OpCode::JLES),
            0x28 => Ok(OpCode::JEQU),
            0x29 => Ok(OpCode::JGRE),
            0x2A => Ok(OpCode::JNLES),
            0x2B => Ok(OpCode::JNEQU),
            0x2C => Ok(OpCode::JNGRE),
            0x31 => Ok(OpCode::CALL),
            0x32 => Ok(OpCode::EXIT),
            0x33 => Ok(OpCode::PUSH),
            0x34 => Ok(OpCode::POP),
            0x35 => Ok(OpCode::PUSHR),
            0x36 => Ok(OpCode::POPR),
            0x70 => Ok(OpCode::SVC),
            // Extended
            0x39 => Ok(OpCode::IEXIT),
            0x71 => Ok(OpCode::HLT),
            0x72 => Ok(OpCode::HCF),
            _ => Err(())
        }
    }
}

impl FromStr for OpCode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NOP" => Ok(OpCode::NOP),
            "STORE" => Ok(OpCode::STORE),
            "LOAD" => Ok(OpCode::LOAD),
            "IN" => Ok(OpCode::IN),
            "OUT" => Ok(OpCode::OUT),
            "ADD" => Ok(OpCode::ADD),
            "SUB" => Ok(OpCode::SUB),
            "MUL" => Ok(OpCode::MUL),
            "DIV" => Ok(OpCode::DIV),
            "MOD" => Ok(OpCode::MOD),
            "AND" => Ok(OpCode::AND),
            "OR" => Ok(OpCode::OR),
            "XOR" => Ok(OpCode::XOR),
            "SHL" => Ok(OpCode::SHL),
            "SHR" => Ok(OpCode::SHR),
            "NOT" => Ok(OpCode::NOT),
            "SHRA" => Ok(OpCode::SHRA),
            "COMP" => Ok(OpCode::COMP),
            "JUMP" => Ok(OpCode::JUMP),
            "JNEG" => Ok(OpCode::JNEG),
            "JZER" => Ok(OpCode::JZER),
            "JPOS" => Ok(OpCode::JPOS),
            "JNNEG" => Ok(OpCode::JNNEG),
            "JNZER" => Ok(OpCode::JNZER),
            "JNPOS" => Ok(OpCode::JNPOS),
            "JLES" => Ok(OpCode::JLES),
            "JEQU" => Ok(OpCode::JEQU),
            "JGRE" => Ok(OpCode::JGRE),
            "JNLES" => Ok(OpCode::JNLES),
            "JNEQU" => Ok(OpCode::JNEQU),
            "JNGRE" => Ok(OpCode::JNGRE),
            "CALL" => Ok(OpCode::CALL),
            "EXIT" => Ok(OpCode::EXIT),
            "PUSH" => Ok(OpCode::PUSH),
            "POP" => Ok(OpCode::POP),
            "PUSHR" => Ok(OpCode::PUSHR),
            "POPR" => Ok(OpCode::POPR),
            "SVC" => Ok(OpCode::SVC),
            // Extended
            "IEXIT" => Ok(OpCode::IEXIT),
            "HLT" => Ok(OpCode::HLT),
            "HCF" => Ok(OpCode::HCF),
            _ => return Err(format!("{} is not an instruction.", s)),
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpCode::NOP => write!(f, "NOP"),
            OpCode::STORE => write!(f, "STORE"),
            OpCode::LOAD => write!(f, "LOAD"),
            OpCode::IN => write!(f, "IN"),
            OpCode::OUT => write!(f, "OUT"),
            OpCode::ADD => write!(f, "ADD"),
            OpCode::SUB => write!(f, "SUB"),
            OpCode::MUL => write!(f, "MUL"),
            OpCode::DIV => write!(f, "DIV"),
            OpCode::MOD => write!(f, "MOD"),
            OpCode::AND => write!(f, "AND"),
            OpCode::OR => write!(f, "OR"),
            OpCode::XOR => write!(f, "XOR"),
            OpCode::SHL => write!(f, "SHL"),
            OpCode::SHR => write!(f, "SHR"),
            OpCode::NOT => write!(f, "NOT"),
            OpCode::SHRA => write!(f, "SHRA"),
            OpCode::COMP => write!(f, "COMP"),
            OpCode::JUMP => write!(f, "JUMP"),
            OpCode::JNEG => write!(f, "JNEG"),
            OpCode::JZER => write!(f, "JZER"),
            OpCode::JPOS => write!(f, "JPOS"),
            OpCode::JNNEG => write!(f, "JNNEG"),
            OpCode::JNZER => write!(f, "JNZER"),
            OpCode::JNPOS => write!(f, "JNPOS"),
            OpCode::JLES => write!(f, "JLES"),
            OpCode::JEQU => write!(f, "JEQU"),
            OpCode::JGRE => write!(f, "JGRE"),
            OpCode::JNLES => write!(f, "JNLES"),
            OpCode::JNEQU => write!(f, "JNEQU"),
            OpCode::JNGRE => write!(f, "JNGRE"),
            OpCode::CALL => write!(f, "CALL"),
            OpCode::EXIT => write!(f, "EXIT"),
            OpCode::PUSH => write!(f, "PUSH"),
            OpCode::POP => write!(f, "POP"),
            OpCode::PUSHR => write!(f, "PUSHR"),
            OpCode::POPR => write!(f, "POPR"),
            OpCode::SVC => write!(f, "SVC"),
            // Extended
            OpCode::IEXIT => write!(f, "IEXIT"),
            OpCode::HLT => write!(f, "HLT"),
            OpCode::HCF => write!(f, "HCF"),
        }
    }
}

impl OpCode {
    /// How many operands does this opcode expect?
    pub fn get_operand_count(&self) -> usize {
        match self {
            OpCode::NOP => 0,
            OpCode::STORE => 2,
            OpCode::LOAD => 2,
            OpCode::IN => 2,
            OpCode::OUT => 2,
            OpCode::ADD => 2,
            OpCode::SUB => 2,
            OpCode::MUL => 2,
            OpCode::DIV => 2,
            OpCode::MOD => 2,
            OpCode::AND => 2,
            OpCode::OR => 2,
            OpCode::XOR => 2,
            OpCode::SHL => 2,
            OpCode::SHR => 2,
            OpCode::NOT => 1,
            OpCode::SHRA => 2,
            OpCode::COMP => 2,
            OpCode::JUMP => 1,
            OpCode::JNEG => 2,
            OpCode::JZER => 2,
            OpCode::JPOS => 2,
            OpCode::JNNEG => 2,
            OpCode::JNZER => 2,
            OpCode::JNPOS => 2,
            OpCode::JLES => 1,
            OpCode::JEQU => 1,
            OpCode::JGRE => 1,
            OpCode::JNLES => 1,
            OpCode::JNEQU => 1,
            OpCode::JNGRE => 1,
            OpCode::CALL => 2,
            OpCode::EXIT => 2,
            OpCode::PUSH => 2,
            OpCode::POP => 2,
            OpCode::PUSHR => 1,
            OpCode::POPR => 1,
            OpCode::SVC => 2,
            // Extended
            OpCode::IEXIT => 2,
            OpCode::HLT => 0,
            OpCode::HCF => 0,
        }
    }

    /// What is the default mode for this opcode?
    /// Usually 1, but some instructions _require_ operating on a memory address, in which case it
    /// is 0.
    pub fn get_default_mode(&self) -> i32 {
        match self {
            OpCode::NOP => 1,
            OpCode::STORE => 0,
            OpCode::LOAD => 1,
            OpCode::IN => 1,
            OpCode::OUT => 1,
            OpCode::ADD => 1,
            OpCode::SUB => 1,
            OpCode::MUL => 1,
            OpCode::DIV => 1,
            OpCode::MOD => 1,
            OpCode::AND => 1,
            OpCode::OR => 1,
            OpCode::XOR => 1,
            OpCode::SHL => 1,
            OpCode::SHR => 1,
            OpCode::NOT => 1,
            OpCode::SHRA => 1,
            OpCode::COMP => 1,
            OpCode::JUMP => 0,
            OpCode::JNEG => 0,
            OpCode::JZER => 0,
            OpCode::JPOS => 0,
            OpCode::JNNEG => 0,
            OpCode::JNZER => 0,
            OpCode::JNPOS => 0,
            OpCode::JLES => 0,
            OpCode::JEQU => 0,
            OpCode::JGRE => 0,
            OpCode::JNLES => 0,
            OpCode::JNEQU => 0,
            OpCode::JNGRE => 0,
            OpCode::CALL => 0,
            OpCode::EXIT => 1,
            OpCode::PUSH => 1,
            OpCode::POP => 1,
            OpCode::PUSHR => 1,
            OpCode::POPR => 1,
            OpCode::SVC => 1,
            // Extended
            OpCode::IEXIT => 1,
            OpCode::HLT => 1,
            OpCode::HCF => 1,
        }
    }

    /// Special case: First operand is _not_ expexted.
    /// Applies to JUMP and State Register using jumps.
    pub fn is_op2_only(&self) -> bool {
        match self {
            OpCode::NOP => false,
            OpCode::STORE => false,
            OpCode::LOAD => false,
            OpCode::IN => false,
            OpCode::OUT => false,
            OpCode::ADD => false,
            OpCode::SUB => false,
            OpCode::MUL => false,
            OpCode::DIV => false,
            OpCode::MOD => false,
            OpCode::AND => false,
            OpCode::OR => false,
            OpCode::XOR => false,
            OpCode::SHL => false,
            OpCode::SHR => false,
            OpCode::NOT => false,
            OpCode::SHRA => false,
            OpCode::COMP => false,
            OpCode::JUMP => true,
            OpCode::JNEG => false,
            OpCode::JZER => false,
            OpCode::JPOS => false,
            OpCode::JNNEG => false,
            OpCode::JNZER => false,
            OpCode::JNPOS => false,
            OpCode::JLES => true,
            OpCode::JEQU => true,
            OpCode::JGRE => true,
            OpCode::JNLES => true,
            OpCode::JNEQU => true,
            OpCode::JNGRE => true,
            OpCode::CALL => false,
            OpCode::EXIT => false,
            OpCode::PUSH => false,
            OpCode::POP => false,
            OpCode::PUSHR => false,
            OpCode::POPR => false,
            OpCode::SVC => false,
            // Extended
            OpCode::IEXIT => false,
            OpCode::HLT => false,
            OpCode::HCF => false,
        }
    }

    /// If you're only interested in the "classic" backwards-compatible instruction set and want to
    /// block or ignore titomachine's extended instructions, you can use this to check.
    pub fn is_classic_isa(&self) -> bool {
        match self {
            OpCode::NOP => true,
            OpCode::STORE => true,
            OpCode::LOAD => true,
            OpCode::IN => true,
            OpCode::OUT => true,
            OpCode::ADD => true,
            OpCode::SUB => true,
            OpCode::MUL => true,
            OpCode::DIV => true,
            OpCode::MOD => true,
            OpCode::AND => true,
            OpCode::OR => true,
            OpCode::XOR => true,
            OpCode::SHL => true,
            OpCode::SHR => true,
            OpCode::NOT => true,
            OpCode::SHRA => true,
            OpCode::COMP => true,
            OpCode::JUMP => true,
            OpCode::JNEG => true,
            OpCode::JZER => true,
            OpCode::JPOS => true,
            OpCode::JNNEG => true,
            OpCode::JNZER => true,
            OpCode::JNPOS => true,
            OpCode::JLES => true,
            OpCode::JEQU => true,
            OpCode::JGRE => true,
            OpCode::JNLES => true,
            OpCode::JNEQU => true,
            OpCode::JNGRE => true,
            OpCode::CALL => true,
            OpCode::EXIT => true,
            OpCode::PUSH => true,
            OpCode::POP => true,
            OpCode::PUSHR => true,
            OpCode::POPR => true,
            OpCode::SVC => true,
            // Extended
            OpCode::IEXIT => false,
            OpCode::HLT => false,
            OpCode::HCF => false,
        }
    }
}

impl FromStr for Register {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "R0" => Ok(Register::R0),
            "R1" => Ok(Register::R1),
            "R2" => Ok(Register::R2),
            "R3" => Ok(Register::R3),
            "R4" => Ok(Register::R4),
            "R5" => Ok(Register::R5),
            "R6" | "SP" => Ok(Register::R6),
            "R7" | "FP" => Ok(Register::R7),
            _ => Err(format!("{} is not a register.", s))
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Register::R0 => write!(f, "R0"),
            Register::R1 => write!(f, "R1"),
            Register::R2 => write!(f, "R2"),
            Register::R3 => write!(f, "R3"),
            Register::R4 => write!(f, "R4"),
            Register::R5 => write!(f, "R5"),
            Register::R6 => write!(f, "SP"),
            Register::R7 => write!(f, "FP"),
        }
    }
}

impl TryFrom<i32> for Register {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::R0),
            1 => Ok(Register::R1),
            2 => Ok(Register::R2),
            3 => Ok(Register::R3),
            4 => Ok(Register::R4),
            5 => Ok(Register::R5),
            6 => Ok(Register::R6),
            7 => Ok(Register::R7),
            _ => Err(()),
        }
    }
}
