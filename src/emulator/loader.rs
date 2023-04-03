/*
 * Funcs here will
 * - Load a program to memory (and set up control regs)
 *
 * Accessing memory through cpu is dumb
 */

use std::str::Lines;

use crate::emulator::devices::MMIO;

use super::{cpu::CPU, devices::Bus};

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
                            Ok(n) => cpu.debug_set_fp(n),
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
                        Ok(n) => cpu.debug_set_sp(n),
                        Err(_) => break,
                    },
                    None => break,
                },
                None => break,
            },
            "___symboltable___" => symbols(cpu, &mut lines),
            "___end___" => {
                // Shouldn't even reach this.
                break;
            }
            _ => match line.parse::<i32>() {
                Ok(value) => {
                    if mem_idx > 0x1fff {
                        println!("ERR: Program does not fit in memory!\nConsider increasing memory size or making smaller programs.\nRan out at address {}.", mem_idx);
                        bus.ram.clear();
                        cpu.debug_clear_cu();
                        return;
                    }
                    bus.write(mem_idx as i32, value).map_err(|err| println!("Loader memory write fail!\n{:?}", err)).ok();
                    mem_idx += 1;
                }
                Err(_e) => {
                    println!("ERR: Failed to parse \"{}\" as a 32bit integer.", line);
                    bus.ram.clear();
                    return;
                }
            },
        }
    }
    cpu.debug_print_regs();
}

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
                    println!("matching {:?}", s);
                    match s {
                        "EXCEPTIONHANDLER0" => cpu.debug_set_ivt(0, value),
                        "EXCEPTIONHANDLER1" => cpu.debug_set_ivt(1, value),
                        "EXCEPTIONHANDLER2" => cpu.debug_set_ivt(2, value),
                        "EXCEPTIONHANDLER3" => cpu.debug_set_ivt(3, value),
                        "EXCEPTIONHANDLER4" => cpu.debug_set_ivt(4, value),
                        "INTERRUPTHANDLER5" => cpu.debug_set_ivt(5, value),
                        "INTERRUPTHANDLER6" => cpu.debug_set_ivt(6, value),
                        "INTERRUPTHANDLER7" => cpu.debug_set_ivt(7, value),
                        "INTERRUPTHANDLER8" => cpu.debug_set_ivt(8, value),
                        "INTERRUPTHANDLER9" => cpu.debug_set_ivt(9, value),
                        "INTERRUPTHANDLER10" => cpu.debug_set_ivt(10, value),
                        "SVC11" => cpu.debug_set_ivt(11, value),
                        "SVC12" => cpu.debug_set_ivt(12, value),
                        "SVC13" => cpu.debug_set_ivt(13, value),
                        "SVC14" => cpu.debug_set_ivt(14, value),
                        "SVC15" => cpu.debug_set_ivt(15, value),
                        _ => continue,
                    }
                }
            }
            None => break,
        }
    }
}
