/*
 * Funcs here will
 * - Load a program to memory (and set up control regs)
 *
 * Accessing memory through cpu is dumb
 */

use super::{
    cpu::CPU,
    cpu::{FP, SP},
};

pub fn load_program(cpu: &mut CPU, prog: &str) {
    cpu.debug_memclear();
    let mut mem_idx: usize = 0;
    let mut lines = prog.lines();
    loop {
        let line;
        match lines.next() {
            None => break,
            Some(s) => line = s,
        }
        match line {
            "___b91___" => {}
            "___code___" => match lines.next() {
                Some(ln) => match ln.split_whitespace().nth(1) {
                    Some(word) => match word.parse::<i32>() {
                        Ok(n) => cpu.debug_set_fp(n),
                        Err(_) => break,
                    },
                    None => break,
                },
                None => break,
            },
            "___data___" => match lines.next() {
                Some(ln) => match ln.split_whitespace().nth(1) {
                    Some(word) => match word.parse::<i32>() {
                        Ok(n) => cpu.debug_set_sp(n),
                        Err(_) => break,
                    },
                    None => break,
                },
                None => break,
            },
            "___symboltable___" => {
                // We don't care about symbols yet.
                break;
            }
            "___end___" => {
                // Shouldn't even reach this.
                break;
            }
            _ => match line.parse::<i32>() {
                Ok(value) => {
                    if mem_idx >= cpu.debug_memlen() {
                        println!("ERR: Program does not fit in memory!\nConsider increasing memory size or making smaller programs.\nRan out at address {}.", cpu.debug_memlen());
                        cpu.debug_memclear();
                        cpu.debug_clear_cu();
                        return;
                    }
                    cpu.debug_memwrite(mem_idx, value);
                    mem_idx += 1;
                }
                Err(_e) => {
                    println!("ERR: Failed to parse \"{}\" as a 32bit integer.", line);
                    cpu.debug_memclear();
                    return;
                }
            },
        }
    }
}
