use super::instance::TTKInstance;

pub fn load_program(prog: &str, instance: &mut TTKInstance) {
    let mut mem_idx: usize = 0;

    // Skip is used to skip lines belo headers.
    let mut skip = 0;
    for line in prog.lines() {
        match line {
            "___b91___" => {
                continue;
            }
            "___code___" => {
                skip += 1;
                continue;
            }
            "___data___" => {
                skip += 1;
                instance.gpr[6] = mem_idx as i32;
                instance.gpr[7] = mem_idx as i32;
                continue;
            }
            "___symboltable___" => {
                // We don't care about symbols yet.
                break;
            }
            "___end___" => {
                // Shouldn't even reach this.
                break;
            }
            _ => {
                if skip > 0 {
                    skip -= 1;
                    continue;
                }
            }
        }
        match line.parse::<i32>() {
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
        }
    }
}

pub fn clear(instance: &mut TTKInstance) {
    for i in &mut instance.memory {
        *i = 0;
    }
    instance.pc = 0;
    instance.ir = 0;
    instance.tr = 0;
    instance.sr = 0;
    for i in &mut instance.gpr {
        *i = 0;
    }
}

pub fn setmemsize(instance: &mut TTKInstance, size: usize) {
    instance.memory.clear();
    instance.memory.resize(size, 0);
}
