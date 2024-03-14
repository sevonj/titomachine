/*
 * Funcs here will
 * - Load a program to memory (and set up control regs)
 *
 */

const SP: usize = 6;
const FP: usize = 7;

use super::{cpu::CPU, devices::Bus};
use crate::emulator::devices::Device;
use std::str::Lines;

pub fn load_program(bus: &mut Bus, cpu: &mut CPU, prog: &str) {
    load(bus, cpu, prog, 0);
}
fn load(bus: &mut Bus, cpu: &mut CPU, prog: &str, org: usize) {
    let mut mem_idx = org;
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
                Some(ln) => {
                    match ln.split_whitespace().nth(0) {
                        Some(word) => match word.parse::<usize>() {
                            Ok(n) => mem_idx += n,
                            Err(_) => break,
                        },
                        None => break,
                    }
                    match ln.split_whitespace().nth(1) {
                        Some(word) => match word.parse::<i32>() {
                            Ok(n) => cpu.debug_set_gpr(FP, n),
                            Err(_) => break,
                        },
                        None => break,
                    }
                }
                None => break,
            },
            "___data___" => match lines.next() {
                Some(ln) => match ln.split_whitespace().nth(1) {
                    Some(word) => match word.parse::<i32>() {
                        Ok(n) => cpu.debug_set_gpr(SP, n),
                        Err(_) => break,
                    },
                    None => break,
                },
                None => break,
            },
            "___symboltable___" => {
                symbols(cpu, &mut lines);
                break;
            }
            "___end___" => panic!("Loader reached ___end___, but it should have stopped before."),
            _ => match line.parse::<i32>() {
                Ok(value) => {
                    if mem_idx > 0x1fff {
                        println!("ERR: Program does not fit in memory!\nConsider increasing memory size or making smaller programs.\nRan out at address {}.", mem_idx);
                        bus.ram.reset();
                        cpu.init();
                        return;
                    }
                    bus.write(mem_idx as u32, value)
                        .map_err(|err| println!("Loader memory write fail!\n{:?}", err))
                        .ok();
                    mem_idx += 1;
                }
                Err(_e) => {
                    println!("ERR: Failed to parse \"{}\" as a 32bit integer.", line);
                    bus.ram.reset();
                    return;
                }
            },
        }
    }
}

/// This fn looks for special symbols.
/// Current usage is IVT entries.
fn symbols(cpu: &mut CPU, lines: &mut Lines) {
    loop {
        match lines.next() {
            Some(ln) => {
                if let Some(s) = ln.split_whitespace().nth(0) {
                    let value;
                    match ln.split_whitespace().nth(1) {
                        Some(word) => match word.parse::<u32>() {
                            Ok(n) => value = n as i32,
                            Err(_) => break,
                        },
                        None => break,
                    }
                    match s {
                        "__IVT_ENTRY_0__" => cpu.debug_set_ivt(0, value),
                        "__IVT_ENTRY_1__" => cpu.debug_set_ivt(1, value),
                        "__IVT_ENTRY_2__" => cpu.debug_set_ivt(2, value),
                        "__IVT_ENTRY_3__" => cpu.debug_set_ivt(3, value),
                        "__IVT_ENTRY_4__" => cpu.debug_set_ivt(4, value),
                        "__IVT_ENTRY_5__" => cpu.debug_set_ivt(5, value),
                        "__IVT_ENTRY_6__" => cpu.debug_set_ivt(6, value),
                        "__IVT_ENTRY_7__" => cpu.debug_set_ivt(7, value),
                        "__IVT_ENTRY_8__" => cpu.debug_set_ivt(8, value),
                        "__IVT_ENTRY_9__" => cpu.debug_set_ivt(9, value),
                        "__IVT_ENTRY_10__" => cpu.debug_set_ivt(10, value),
                        "__IVT_ENTRY_11__" => cpu.debug_set_ivt(11, value),
                        "__IVT_ENTRY_12__" => cpu.debug_set_ivt(12, value),
                        "__IVT_ENTRY_13__" => cpu.debug_set_ivt(13, value),
                        "__IVT_ENTRY_14__" => cpu.debug_set_ivt(14, value),
                        "__IVT_ENTRY_15__" => cpu.debug_set_ivt(15, value),
                        _ => continue,
                    }
                }
            }
            None => break,
        }
    }
}
