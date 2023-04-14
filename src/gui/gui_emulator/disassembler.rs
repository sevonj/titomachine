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
    IEXIT = 0x39,
    SVC = 0x70,
    HLT = 0x71,
    HCF = 0x72,
}

fn second2string(m: i32, ri: i32, addr: i32) -> String {
    let mut str = String::new();
    let mut m = m;
    if addr == 0 && ri != 0 {
        m += 1;
    }
    match m {
        0 => str += "=",
        1 => str += " ",
        2 => str += "@",
        3 => {
            // @(R1) results to this
            return format!("@({})", reg2string(ri));
        }
        _ => str += "wtf‽",
    }
    if ri == 0 {
        str += &addr.to_string();
        return str;
    }
    if addr != 0 {
        str += &addr.to_string();
        str += "(";
    }
    str += reg2string(ri).as_str();
    if addr != 0 {
        str += ")";
    }
    str
}

fn reg2string(r: i32) -> String {
    match r {
        6 => "SP, ".into(),
        7 => "FP, ".into(),
        _ => format!("R{}", r),
    }
}

fn rj2string(r: i32) -> String {
    match r {
        6 => "SP, ".into(),
        7 => "FP, ".into(),
        _ => format!("R{}, ", r),
    }
}

pub fn instruction_to_string(input_instr: i32) -> String {
    let mut retstr = String::new();

    let opcode = input_instr >> 24;
    let rj = (input_instr >> 21) & 0x7;
    let mode = (input_instr >> 19) & 0x3;
    let ri = (input_instr >> 16) & 0x7;
    let addr = (input_instr & 0xffff) as i16 as i32; // these casts catch the sign

    match FromPrimitive::from_i32(opcode) {
        Some(Opcode::NOP) => retstr += "NOP",
        Some(Opcode::STORE) => {
            retstr += "STORE ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::LOAD) => {
            retstr += "LOAD  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::IN) => {
            retstr += "IN    ";
            retstr += rj2string(rj).as_str();
            println!("IN mode: {}", mode);
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::OUT) => {
            retstr += "OUT   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::ADD) => {
            retstr += "ADD   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::SUB) => {
            retstr += "SUB   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::MUL) => {
            retstr += "MUL   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::DIV) => {
            retstr += "DIV   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::MOD) => {
            retstr += "MOD   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::AND) => {
            retstr += "AND   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::OR) => {
            retstr += "OR    ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::XOR) => {
            retstr += "XOR   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::SHL) => {
            retstr += "SHL   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::SHR) => {
            retstr += "SHR   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::NOT) => {
            retstr += "NOT   ";
            retstr += rj2string(rj).as_str();
        }
        Some(Opcode::SHRA) => {
            retstr += "SHRA  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::COMP) => {
            retstr += "COMP  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::JUMP) => {
            retstr += "JUMP  ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JNEG) => {
            retstr += "JNEG  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JZER) => {
            retstr += "JZER  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JPOS) => {
            retstr += "JPOS ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JNNEG) => {
            retstr += "JNNEG ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JNZER) => {
            retstr += "JNZER ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JNPOS) => {
            retstr += "JNPOS ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JLES) => {
            retstr += "JLES  ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JEQU) => {
            retstr += "JEQU  ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JGRE) => {
            retstr += "JGRE  ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JNLES) => {
            retstr += "JNLES ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JNEQU) => {
            retstr += "JNEQU ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::JNGRE) => {
            retstr += "JNGRE ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::CALL) => {
            retstr += "CALL  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(Opcode::EXIT) => {
            retstr += "EXIT  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::PUSH) => {
            retstr += "PUSH  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::POP) => {
            retstr += "POP   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::PUSHR) => {
            retstr += "PUSHR ";
            retstr += rj2string(rj).as_str();
        }
        Some(Opcode::POPR) => {
            retstr += "POPR  ";
            retstr += rj2string(rj).as_str();
        }
        Some(Opcode::IEXIT) => {
            retstr += "IEXIT ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::SVC) => {
            retstr += "SVC   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(Opcode::HLT) => retstr += "HLT   ",
        Some(Opcode::HCF) => retstr += "HCF   ",

        None => retstr += "INVALID",
    }

    retstr
}
