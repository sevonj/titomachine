use crate::emulator::instructions::{Opcode, TTK91Instruction};
use num_traits::FromPrimitive;

fn sec_operand(ins: TTK91Instruction) -> String {
    let mut str = String::new();

    if ins.ri == 0 {
        return ins.addr.to_string();
    }

    if ins.addr != 0 {
        str += &ins.addr.to_string();
        str += "(";
    }

    match ins.ri {
        6 => str += "SP",
        7 => str += "FP",
        _ => {
            str += "R";
            str += &ins.ri.to_string();
        }
    }
    if ins.addr != 0 {
        str += ")";
    }

    str
}

fn rj(r: i32) -> String {
    let mut str = String::new();
    match r {
        6 => str += "SP",
        7 => str += "FP",
        _ => {
            str += "R";
            str += &r.to_string();
        }
    }
    str += ", ";
    str
}

fn mode(m: i32) -> String {
    let mut str = String::new();
    match m {
        0 => str += "=",
        1 => str += " ",
        2 => str += "@",
        _ => str += "wtf‽",
    }
    str
}

fn ri(r: i32) -> String {
    let mut str = String::new();
    match r {
        0 => {}
        6 => str += "SP",
        7 => str += "FP",
        _ => {
            str += "R";
            str += &r.to_string();
        }
    }
    str
}

pub fn instruction_to_string(input_instr: i32) -> String {
    let mut retstr = String::new();

    let ins = TTK91Instruction::from(input_instr);

    match FromPrimitive::from_i32(ins.opcode) {
        Some(Opcode::NOP) => retstr += "NOP",
        Some(Opcode::STORE) => {
            retstr += "STORE ";
            retstr += rj(ins.rj).as_str();
            match ins.mode {
                0 => retstr += " ",
                1 => retstr += "@",
                2 => retstr += "wtf‽", // indirect is forbidden by compiler.
                _ => retstr += "wtf‽",
            }
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::LOAD) => {
            retstr += "LOAD  ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::IN) => {
            retstr += "IN    ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::OUT) => {
            retstr += "OUT   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::ADD) => {
            retstr += "ADD   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::SUB) => {
            retstr += "SUB   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::MUL) => {
            retstr += "MUL   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::DIV) => {
            retstr += "DIV   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::MOD) => {
            retstr += "MOD   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::AND) => {
            retstr += "AND   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::OR) => {
            retstr += "OR    ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::XOR) => {
            retstr += "XOR   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::SHL) => {
            retstr += "SHL   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::SHR) => {
            retstr += "SHR   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::NOT) => {
            retstr += "NOT   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::SHRA) => {
            retstr += "SHRA  ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::COMP) => {
            retstr += "COMP  ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JUMP) => {
            retstr += "JUMP  ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JNEG) => {
            retstr += "JNEG  ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JZER) => {
            retstr += "JZER  ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JPOS) => {
            retstr += "JPOS ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JNNEG) => {
            retstr += "JNNEG ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JNZER) => {
            retstr += "JNZER ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JNPOS) => {
            retstr += "JNPOS ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JLES) => {
            retstr += "JLES  ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JEQU) => {
            retstr += "JEQU  ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JGRE) => {
            retstr += "JGRE  ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JNLES) => {
            retstr += "JNLES ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JNEQU) => {
            retstr += "JNEQU ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::JNGRE) => {
            retstr += "JNGRE ";
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::CALL) => {
            retstr += "CALL  ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::EXIT) => {
            retstr += "EXIT  ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::PUSH) => {
            retstr += "PUSH  ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::POP) => {
            retstr += "POP   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        Some(Opcode::PUSHR) => {
            retstr += "PUSHR ";
            retstr += rj(ins.rj).as_str();
        }
        Some(Opcode::POPR) => {
            retstr += "POPR  ";
            retstr += rj(ins.rj).as_str();
        }
        Some(Opcode::SVC) => {
            retstr += "SVC   ";
            retstr += rj(ins.rj).as_str();
            retstr += mode(ins.mode).as_str();
            retstr += sec_operand(ins).as_str();
        }
        None => retstr += "INVALID",
    }

    retstr
}
