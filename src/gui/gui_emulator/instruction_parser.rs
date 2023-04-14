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

fn addr2string(ri: i32, addr: i32) -> String {
    let mut str = String::new();
    if ri == 0 {
        return addr.to_string();
    }
    if addr != 0 {
        str += &addr.to_string();
        str += "(";
    }
    match ri {
        6 => str += "SP",
        7 => str += "FP",
        _ => str += format!("R{}", ri).as_str(),
    }
    if addr != 0 {
        str += ")";
    }
    str
}

fn rj2string(r: i32) -> String {
    match r {
        6 => "SP, ".into(),
        7 => "FP, ".into(),
        _ => format!("R{}, ", r),
    }
}

fn mode2string(m: i32) -> String {
    match m {
        0 => "=".into(),
        1 => " ".into(),
        2 => "@".into(),
        _ => "wtf‽".into(),
    }
}

fn ri2string(r: i32) -> String {
    match r {
        0 => "".into(),
        6 => "SP".into(),
        7 => "FP".into(),
        _ => format!("R{}", r),
    }
}

pub fn instruction_to_string(input_instr: i32) -> String {
    let mut retstr = String::new();

    let opcode = input_instr >> 24;
    let rj = (input_instr >> 21) & 0x7;
    let mode; // = (input_instr >> 19) & 0x3;
    let ri = (input_instr >> 16) & 0x7;
    let addr = (input_instr & 0xffff) as i16 as i32; // these casts catch the sign

    match addr == 0 {
        true => mode = ((input_instr >> 19) & 0x3),
        false => mode = ((input_instr >> 19) & 0x3) - 1,
    }

    match FromPrimitive::from_i32(opcode) {
        Some(Opcode::NOP) => retstr += "NOP",
        Some(Opcode::STORE) => {
            retstr += "STORE ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode + 1).as_str(); // Notice +1
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::LOAD) => {
            retstr += "LOAD  ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::IN) => {
            retstr += "IN    ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::OUT) => {
            retstr += "OUT   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::ADD) => {
            retstr += "ADD   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::SUB) => {
            retstr += "SUB   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::MUL) => {
            retstr += "MUL   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::DIV) => {
            retstr += "DIV   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::MOD) => {
            retstr += "MOD   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::AND) => {
            retstr += "AND   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::OR) => {
            retstr += "OR    ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::XOR) => {
            retstr += "XOR   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::SHL) => {
            retstr += "SHL   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::SHR) => {
            retstr += "SHR   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::NOT) => {
            retstr += "NOT   ";
            retstr += rj2string(rj).as_str();
        }
        Some(Opcode::SHRA) => {
            retstr += "SHRA  ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::COMP) => {
            retstr += "COMP  ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JUMP) => {
            retstr += "JUMP  ";
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JNEG) => {
            retstr += "JNEG  ";
            retstr += rj2string(rj).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JZER) => {
            retstr += "JZER  ";
            retstr += rj2string(rj).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JPOS) => {
            retstr += "JPOS ";
            retstr += rj2string(rj).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JNNEG) => {
            retstr += "JNNEG ";
            retstr += rj2string(rj).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JNZER) => {
            retstr += "JNZER ";
            retstr += rj2string(rj).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JNPOS) => {
            retstr += "JNPOS ";
            retstr += rj2string(rj).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JLES) => {
            retstr += "JLES  ";
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JEQU) => {
            retstr += "JEQU  ";
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JGRE) => {
            retstr += "JGRE  ";
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JNLES) => {
            retstr += "JNLES ";
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JNEQU) => {
            retstr += "JNEQU ";
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::JNGRE) => {
            retstr += "JNGRE ";
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::CALL) => {
            retstr += "CALL  ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode + 1).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::EXIT) => {
            retstr += "EXIT  ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::PUSH) => {
            retstr += "PUSH  ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::POP) => {
            retstr += "POP   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode + 1).as_str();
            retstr += addr2string(ri, addr).as_str();
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
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::SVC) => {
            retstr += "SVC   ";
            retstr += rj2string(rj).as_str();
            retstr += mode2string(mode).as_str();
            retstr += addr2string(ri, addr).as_str();
        }
        Some(Opcode::HLT) => retstr += "HLT   ",
        Some(Opcode::HCF) => retstr += "HCF   ",

        None => retstr += "INVALID",
    }

    retstr
}
