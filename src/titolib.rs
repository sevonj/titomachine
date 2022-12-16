/*
 * titolib.rs
 * Titolib bindings and useful functions.
 *
 */

use std::ffi::c_char;
use std::ffi::CString;

// C bindings
extern "C" {
    fn titomach_clear_mem();
    fn titomach_load_prog(filename: *const c_char);
    fn titomach_start();
    fn titomach_exec() -> i32;
    fn titomach_stop();
    fn titomach_debug_read_mem(address: i32) -> i32;
    fn titomach_debug_read_reg(address: i32) -> i32;
    fn titomach_debug_read_cureg(address: i32) -> i32;
    //fn titomach_debug_write_mem(int address, int value);
    fn titomach_is_waiting_for_input() -> i32;
    fn titomach_input(val: i32);
    fn titomach_output_len() -> i32;
    fn titomach_output(idx: i32) -> i32;
}

pub fn tito_readmem(address: i32) -> i32 {
    unsafe {
        let val = titomach_debug_read_mem(address);
        return val;
    }
}
pub fn tito_readreg(idx: i32) -> i32 {
    unsafe {
        let val = titomach_debug_read_reg(idx);
        return val;
    }
}
pub fn tito_readcureg(idx: i32) -> i32 {
    unsafe {
        let val = titomach_debug_read_cureg(idx);
        return val;
    }
}
pub fn tito_clearmem() {
    unsafe {
        titomach_clear_mem();
    }
}
pub fn tito_loadprog(filepath: &str) {
    unsafe {
        titomach_load_prog(convert_str(filepath));
    }
}
pub fn tito_start() {
    unsafe {
        titomach_start();
    }
}
pub fn tito_tick() {
    unsafe {
        titomach_exec();
    }
}
pub fn tito_stop() {
    unsafe {
        titomach_stop();
    }
}
pub fn tito_is_waiting_for_input() -> bool {
    unsafe {
        if titomach_is_waiting_for_input() == 0 {
            return false;
        }
        return true;
    }
}
pub fn tito_input(val: i32) {
    unsafe {
        titomach_input(val);
    }
}
pub fn tito_output_buffer() -> String {
    unsafe {
        let len = titomach_output_len();
        let mut result = String::new();
        for i in 0..16 {
            if i < len {
                result.push_str(format!("{}", titomach_output(i)).as_str());
            }
            result.push_str("\n")
        }
        result
    }
}

// Todo: find out if this is even needed
pub unsafe fn convert_str(input: &str) -> *mut c_char {
    let c_str = CString::new(input).unwrap().into_raw();
    return c_str;
}

// useful functions

pub fn tito_inst_to_string(instruction: i32) -> String {
    let mut result = String::new();
    let opcode = (instruction >> 24) & 0xff;
    let rj = (instruction >> 21) & 0x7;
    let mut mode = (instruction >> 19) & 0x3;
    let ri = (instruction >> 16) & 0x7;
    let address = instruction & 0xffff;

    // sometimes
    let mut hide_rj: bool = false;

    match opcode {
        0 => {
            result.push_str("NOP    ");
            return result;
        }
        1 => {
            result.push_str("STORE  ");
            mode += 1; // "Immediate mode" isn't valid.
                       // But because the value here is the address,
                       // direct becomes immediate and indirect becomes direct.
        }
        2 => result.push_str("LOAD   "),
        3 => result.push_str("IN     "),
        4 => result.push_str("OUT    "),

        17 => result.push_str("ADD    "),
        18 => result.push_str("SUB    "),
        19 => result.push_str("MUL    "),
        20 => result.push_str("DIV    "),
        21 => result.push_str("MOD    "),
        22 => result.push_str("AND    "),
        23 => result.push_str("OR     "),
        24 => result.push_str("XOR    "),
        25 => result.push_str("SHL    "),
        26 => result.push_str("SHR    "),
        27 => result.push_str("NOT    "),
        28 => result.push_str("SHRA   "),

        31 => result.push_str("COMP   "),
        32 => {
            result.push_str("JUMP   ");
            hide_rj = true;
        }
        33 => {
            result.push_str("JNEG   ");
        }
        34 => {
            result.push_str("JZER   ");
        }
        35 => {
            result.push_str("JPOS   ");
        }
        36 => {
            result.push_str("JNNEG  ");
        }
        37 => {
            result.push_str("JNZER  ");
        }
        38 => {
            result.push_str("JNPOS  ");
        }
        39 => {
            result.push_str("JLES   ");
            hide_rj = true;
        }
        40 => {
            result.push_str("JEQU   ");
            hide_rj = true;
        }
        41 => {
            result.push_str("JGRE   ");
            hide_rj = true;
        }
        42 => {
            result.push_str("JNLES  ");
            hide_rj = true;
        }
        43 => {
            result.push_str("JNEQU  ");
            hide_rj = true;
        }
        44 => {
            result.push_str("JNGRE  ");
            hide_rj = true;
        }

        49 => result.push_str("CALL   "),
        50 => result.push_str("EXIT   "),
        51 => result.push_str("PUSH   "),
        52 => result.push_str("POP    "),
        53 => result.push_str("PUSHR  "),
        54 => result.push_str("POPR   "),

        112 => result.push_str("SVC    "),
        _ => {
            result.push_str("INVALID");
            return result;
        }
    }
    if !hide_rj {
        if rj == 6 {
            result.push_str(&format!("SP "));
        } else if rj == 7 {
            result.push_str(&format!("FP "));
        } else {
            result.push_str(&format!("R{} ", rj));
        }
    }

    if true {
        // !hide_mode { // I don't think this is needed
        if mode == 0 {
            result.push_str("=")
        } else if mode == 1 {
            result.push_str(" ")
        } else {
            result.push_str("@")
        }
    }

    result.push_str(&format!("{}", address));

    if ri == 0 {
        return result;
    }
    if ri == 6 {
        result.push_str(&format!("(SP)"));
    } else if ri == 7 {
        result.push_str(&format!("(FP)"));
    } else {
        result.push_str(&format!("(R{})", ri));
    }
    return result;
}
