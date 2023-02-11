use super::instance::{TTKInstance, FP, SP};

pub fn load_program(prog: &str, instance: &mut TTKInstance) {
    let mut mem_idx: usize = 0;

    // Skip is used to skip lines belo headers.
    let mut skip = 0;
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
                        Ok(n) => instance.gpr[FP] = n,
                        Err(_) => break,
                    },
                    None => break,
                },
                None => break,
            },
            "___data___" => match lines.next() {
                Some(ln) => match ln.split_whitespace().nth(1) {
                    Some(word) => match word.parse::<i32>() {
                        Ok(n) => instance.gpr[SP] = n,
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
                    if mem_idx >= instance.memory.len() {
                        println!("ERR: Program does not fit in memory!\nConsider increasing memory size or making smaller programs.\nRan out at address {}.", instance.memory.len());
                        clear(instance);
                        return;
                    }
                    instance.memory[mem_idx] = value;
                    mem_idx += 1;
                }
                Err(_e) => {
                    println!("ERR: Failed to parse \"{}\" as a 32bit integer.", line);
                    clear(instance);
                    return;
                }
            },
        }
    }
}

pub fn clear(instance: &mut TTKInstance) {
    for i in &mut instance.memory {
        *i = 0;
    }
    instance.cu_pc = 0;
    instance.cu_ir = 0;
    instance.cu_tr = 0;
    instance.cu_sr = 0;
    for i in &mut instance.gpr {
        *i = 0;
    }
}

pub fn setmemsize(instance: &mut TTKInstance, size: usize) {
    instance.memory.clear();
    instance.memory.resize(size, 0);
}
