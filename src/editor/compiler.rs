use std::collections::HashMap;

pub fn compile(source: String) -> String {
    let mut var_symbols: Vec<String> = Vec::new();
    let mut symbols_table: HashMap<String, i32> = HashMap::from([
        ("CRT".into(), 0),
        ("KBD".into(), 1),
        ("HALT".into(), 11),
        ("READ".into(), 12),
        ("WRITE".into(), 13),
        ("TIME".into(), 14),
        ("DATE".into(), 15),
    ]);

    let mut data: Vec<i32> = Vec::new();
    let mut prog: Vec<i32> = Vec::new();

    // instructions.push(TTK91Instruction { opcode: 1, rj: 1, mode: 1, ri: 1, addr: 1 });

    // Process lines: remove anything unnecessary
    let mut processed_lines: Vec<String> = Vec::new();
    for line in source.lines() {
        let mut result_line = String::new();
        // empty line
        if line == "" {
            processed_lines.push("".into());
            continue;
        }
        // Remove comments
        result_line += line.split(";").take(1).next().unwrap();

        // Commas to spaces
        result_line = result_line.replace(",", " ");
        // Split words
        let words: Vec<String> = result_line.split_whitespace().map(str::to_string).collect();
        result_line = words.join(" ");
        // Case
        result_line = result_line.to_uppercase();
        // Push
        processed_lines.push(result_line);
    }

    // get constants
    let mut ln = 0;
    for line in &processed_lines {
        ln += 1;
        if line.len() == 0 {
            continue;
        }
        let words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        if match_instruction(words[0].as_str()) == -1 {
            if words.len() < 2 {
                // No word after symbol
                println!("Line {}: No instruction!", ln);
                return "".into();
            }
            if match_pseudoinstr(words[1].as_str()) == -1
                && match_instruction(words[1].as_str()) == -1
            {
                // Not an instruction nor a pseudoinstruction
                println!("Line {}: Invalid instruction!", ln);
                return "".into();
            }
            if words[1].as_str() == "EQU" {
                // Check for no value
                if words.len() < 3 {
                    println!("Line {}: No value given for constant {}!", ln, words[0]);
                    return "".into();
                }
                let value;
                // Get value
                match parse_number(words[2].as_str()) {
                    Some(n) => value = n,
                    None => {
                        println!("Line {}: Failed to parse constant {}!", ln, words[0]);
                        return "".into();
                    }
                }
                // Check for redefine
                if symbols_table.contains_key(&words[0]) {
                    println!("Line {}: Symbol {} redefined!", ln, words[0]);
                    return "".into();
                }
                // Commit
                println!(
                    "Line {}: Found const {} with value {}.",
                    ln, words[0], value
                );
                symbols_table.insert(words[0].clone(), value);
            }
        }
    }
    // get variables
    ln = 0;
    let mut data_size = 0;
    for line in &processed_lines {
        ln += 1;
        if line.len() == 0 {
            continue;
        }
        let words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        if match_instruction(words[0].as_str()) == -1 {
            if words[1].as_str() == "DC" {
                // Check for no value
                if words.len() < 3 {
                    println!("Line {}: No value given for variable {}!", ln, words[0]);
                    return "".into();
                }
                let value;
                // Get value
                match parse_number(words[2].as_str()) {
                    Some(n) => value = n,
                    None => {
                        println!("Line {}: Failed to parse variable {}!", ln, words[0]);
                        return "".into();
                    }
                }
                // Check for redefine
                if symbols_table.contains_key(&words[0]) {
                    println!("Line {}: Symbol {} redefined!", ln, words[0]);
                    return "".into();
                }
                // Commit
                println!("Line {}: Found Var {} with value {}.", ln, words[0], value);
                data.push(value);
                symbols_table.insert(words[0].clone(), data_size);
                var_symbols.push(words[0].clone());
                data_size += 1;
            } else if words[1].as_str() == "DS" {
                // Check for no value
                if words.len() < 3 {
                    println!("Line {}: No size given for segment {}!", ln, words[0]);
                }
                let value;
                // Get value
                match parse_number(words[2].as_str()) {
                    Some(n) => value = n,
                    None => {
                        println!("Line {}: Failed to parse variable {}!", ln, words[0]);
                        return "".into();
                    }
                }
                // Check for redefine
                if symbols_table.contains_key(&words[0]) {
                    println!("Line {}: Symbol {} redefined!", ln, words[0]);
                    return "".into();
                }
                // Check for positive
                if value < 1 {
                    println!(
                        "Line {}: Cannot reserve zero or negative amount of addresses!",
                        ln
                    );
                    return "".into();
                }
                // Commit
                println!(
                    "Line {}: Found segment {} with size {}.",
                    ln, words[0], value
                );
                for _ in 0..value {
                    data.push(0);
                }
                symbols_table.insert(words[0].clone(), data_size);
                var_symbols.push(words[0].clone());
                data_size += value;
            }
        }
    }

    // get instruction labels
    ln = 0;
    let mut prog_size = 0;
    for line in &processed_lines {
        ln += 1;
        if line.len() == 0 {
            continue;
        }
        let words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        // Is labeled

        if match_instruction(words[0].as_str()) == -1 {
            if words.len() < 2 {
                return "".into();
            }
            if match_instruction(words[1].as_str()) == -1 {
                continue;
            }
            // Is labeled and an instruction
            if symbols_table.contains_key(&words[0]) {
                println!("Line {}: Symbol {} redefined!", ln, words[0]);
                return "".into();
            }
            println!("Line {}: Found label {}.", ln, words[0]);
            symbols_table.insert(words[0].clone(), prog_size);
            prog_size += 1;
        } else {
            // Non labeled instruction
            prog_size += 1;
        }
    }

    for s in var_symbols {
        symbols_table
            .entry(s)
            .and_modify(|offset| *offset += prog_size);
    }

    // Get Instructions
    let mut ln = 0;
    let mut off = 0;
    for line in &processed_lines {
        ln += 1;
        if line.len() == 0 {
            continue;
        }

        let mut words: Vec<String> = line.split_whitespace().map(str::to_string).collect();

        // 1st word is not an opcode
        if match_instruction(words[0].as_str()) == -1 {
            // 2nd word is an opcode
            if match_instruction(words[1].as_str()) != -1 {
                // Remove label
                words = line
                    .split_whitespace()
                    .skip(1)
                    .map(str::to_string)
                    .collect();
            }
            // This line doesn't contain an instruction
            else {
                continue;
            }
        }
        // 1st word is now an opcode, regardless of whether there was a label
        // encode instruction
        let opcode_str = &words[0];
        let op1: &str;
        let op2: &str;

        let opcode: i32;
        let rj: i32;
        let mut mode: i32 = 1;
        let ri: i32;
        let addr: i32;

        // indirect memory access.
        // Normally addressing mode goes like this:
        // "=" => 0,
        // " " => 1,
        // "@" => 2
        // with some instructions indirect addressign is disabled
        // "=" is not allowed,
        // " " => 0,
        // "@" => 1

        match opcode_str.as_str() {
            "NOP" => {
                if words.len() != 1 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x00;
                op1 = "";
                op2 = "";
            }
            "STORE" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x01;
                op1 = &words[1];
                op2 = &words[2];
                mode -= 1; // no indirect
            }
            "LOAD" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x02;
                op1 = &words[1];
                op2 = &words[2];
            }
            "IN" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x03;
                op1 = &words[1];
                op2 = &words[2];
            }
            "OUT" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x04;
                op1 = &words[1];
                op2 = &words[2];
            }
            "ADD" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x11;
                op1 = &words[1];
                op2 = &words[2];
            }
            "SUB" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x12;
                op1 = &words[1];
                op2 = &words[2];
            }
            "MUL" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x13;
                op1 = &words[1];
                op2 = &words[2];
            }
            "DIV" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x14;
                op1 = &words[1];
                op2 = &words[2];
            }
            "MOD" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x15;
                op1 = &words[1];
                op2 = &words[2];
            }
            "AND" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x16;
                op1 = &words[1];
                op2 = &words[2];
            }
            "OR" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x17;
                op1 = &words[1];
                op2 = &words[2];
            }
            "XOR" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x18;
                op1 = &words[1];
                op2 = &words[2];
            }
            "SHL" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x19;
                op1 = &words[1];
                op2 = &words[2];
            }
            "SHR" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x1a;
                op1 = &words[1];
                op2 = &words[2];
            }
            "NOT" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x1b;
                op1 = &words[1];
                op2 = "";
            }
            "SHRA" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x1c;
                op1 = &words[1];
                op2 = &words[2];
            }
            "COMP" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x1f;
                op1 = &words[1];
                op2 = &words[2];
            }
            "JUMP" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x20;
                op1 = "";
                op2 = &words[1];
                mode -= 1; // no indirect
            }
            "JNEG" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x21;
                op1 = &words[1];
                op2 = &words[2];
                mode -= 1; // no indirect
            }
            "JZER" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x22;
                op1 = &words[1];
                op2 = &words[2];
                mode -= 1; // no indirect
            }
            "JPOS" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x23;
                op1 = &words[1];
                op2 = &words[2];
                mode -= 1; // no indirect
            }
            "JNNEG" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x24;
                op1 = &words[1];
                op2 = &words[2];
                mode -= 1; // no indirect
            }
            "JNZER" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x25;
                op1 = &words[1];
                op2 = &words[2];
                mode -= 1; // no indirect
            }
            "JNPOS" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x26;
                op1 = &words[1];
                op2 = &words[2];
                mode -= 1; // no indirect
            }
            "JLES" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x27;
                op1 = "";
                op2 = &words[1];
                mode -= 1; // no indirect
            }
            "JEQU" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x28;
                op1 = "";
                op2 = &words[1];
                mode -= 1; // no indirect
            }
            "JGRE" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x29;
                op1 = "";
                op2 = &words[1];
                mode -= 1; // no indirect
            }
            "JNLES" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x2a;
                op1 = "";
                op2 = &words[1];
                mode -= 1; // no indirect
            }
            "JNEQU" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x2b;
                op1 = "";
                op2 = &words[1];
                mode -= 1; // no indirect
            }
            "JNGRE" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x2c;
                op1 = "";
                op2 = &words[1];
                mode -= 1; // no indirect
            }
            "CALL" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x31;
                op1 = &words[1];
                op2 = &words[2];
                mode -= 1; // no indirect
            }
            "EXIT" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x32;
                op1 = &words[1];
                op2 = &words[2];
            }
            "PUSH" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x33;
                op1 = &words[1];
                op2 = &words[2];
            }
            "POP" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x34;
                op1 = &words[1];
                op2 = &words[2];
                if match_reg(op2) == -1 {
                    println!("Second operand must be a register for POP");
                    return "".into();
                }
            }
            "PUSHR" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x35;
                op1 = &words[1];
                op2 = "";
            }
            "POPR" => {
                if words.len() != 2 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x36;

                op1 = &words[1];
                op2 = "0";
            }
            "SVC" => {
                if words.len() != 3 {
                    println!("Line {} Unacceptable amount of terms!", ln);
                    return "".into();
                }
                opcode = 0x70;
                op1 = &words[1];
                op2 = &words[2];
            }
            _ => {
                println!("Something is wrong with the compiler :(");
                return "".into();
            }
        }

        if op1 == "" {
            rj = 0;
        } else {
            rj = match_reg(op1);
            if rj == -1 {
                println!("Invalid register on line {}!", ln);
                return "".into();
            }
        }
        if op2 == "" {
            mode = 1;
            ri = 0;
            addr = 0;
        } else {
            // Mode

            if op2.matches(['@', '=']).count() > 1 {
                println!("Invalid addressing mode on line {}!", ln);
                return "".into();
            }

            mode -= op2.matches(['=']).count() as i32;
            mode += op2.matches(['@']).count() as i32;

            if mode < 0 {
                println!("Invalid addressing mode on line {}!", ln);
                return "".into();
            }

            let op2_wo_mode = op2.replace(['@', '='], "");

            // No addr, only reg
            if match_reg(&op2_wo_mode) != -1 {
                // Disallow =R1, @R1
                mode -= 1; // no indirect
                if mode < 0 {
                    println!("Invalid addressing mode on line {}!", ln);
                    return "".into();
                }

                ri = match_reg(&op2_wo_mode);
                addr = 0;
            } else {
                // Addr
                let addr_str: String = op2_wo_mode.split("(").take(1).collect();
                if symbols_table.contains_key(&addr_str) {
                    addr = symbols_table[&addr_str];
                } else {
                    match parse_number(addr_str.as_str()) {
                        Some(int) => addr = int,
                        None => {
                            println!("invalid address: {}", addr_str);
                            return "".into();
                        }
                    }
                }

                let mut ri_str: String = op2_wo_mode.split("(").skip(1).collect();
                if ri_str.len() != 0 {
                    ri_str = ri_str.split(")").take(1).collect();
                    ri = match_reg(&ri_str);
                    if ri == -1 {
                        println!("invalid register");
                        return "".into();
                    }
                } else {
                    ri = 0
                }
            }
        }
        //println!("{}, {}, {}, {}, {}, ", opcode, rj, mode, ri, addr);

        let mut instruction: i32 = 0;
        instruction += opcode << 24;
        instruction += rj << 21;
        instruction += mode << 19;
        instruction += ri << 16;
        match i16::try_from(addr) {
            Ok(int) => {
                instruction += (int as i32) & 0xffff;
            }
            Err(e) => println!("Line {}: {}", ln, e),
        }

        prog.push(instruction);

        off += 1;
    }

    let prog_start = 0;
    let fp_start = prog_size-1;
    let data_start = prog_size;
    let sp_start = fp_start + data_size;

    let mut return_str = String::new();
    return_str += "___b91___\n";
    return_str += "___code___\n";
    return_str += (prog_start).to_string().as_str();
    return_str += " ";
    return_str += (fp_start).to_string().as_str();
    return_str += "\n";
    for i in prog {
        return_str += i.to_string().as_str();
        return_str += "\n"
    }
    return_str += "___data___\n";
    return_str += (data_start).to_string().as_str();
    return_str += " ";
    return_str += (sp_start).to_string().as_str();
    return_str += "\n";
    for i in data {
        return_str += i.to_string().as_str();
        return_str += "\n"
    }
    return_str += "___symboltable___\n";
    for (key, value) in symbols_table {
        return_str += key.as_str();
        return_str += " ";
        return_str += value.to_string().as_str();
        return_str += "\n";
    }
    return_str += "___end___\n";

    println!("Compiled:\n{}", return_str);
    return_str
}

fn match_instruction(opstr: &str) -> i32 {
    match opstr {
        "NOP" => 0x00,
        "STORE" => 0x01,
        "LOAD" => 0x02,
        "IN" => 0x03,
        "OUT" => 0x04,
        "ADD" => 0x11,
        "SUB" => 0x12,
        "MUL" => 0x13,
        "DIV" => 0x14,
        "MOD" => 0x15,
        "AND" => 0x16,
        "OR" => 0x17,
        "XOR" => 0x18,
        "SHL" => 0x19,
        "SHR" => 0x1A,
        "NOT" => 0x1B,
        "SHRA" => 0x1C,
        "COMP" => 0x1F,
        "JUMP" => 0x20,
        "JNEG" => 0x21,
        "JZER" => 0x22,
        "JPOS" => 0x23,
        "JNNEG" => 0x24,
        "JNZER" => 0x25,
        "JNPOS" => 0x26,
        "JLES" => 0x27,
        "JEQU" => 0x28,
        "JGRE" => 0x29,
        "JNLES" => 0x2A,
        "JNEQU" => 0x2B,
        "JNGRE" => 0x2C,
        "CALL" => 0x31,
        "EXIT" => 0x32,
        "PUSH" => 0x33,
        "POP" => 0x34,
        "PUSHR" => 0x35,
        "POPR" => 0x36,
        "SVC" => 0x70,
        _ => -1,
    }
}
fn match_pseudoinstr(opstr: &str) -> i32 {
    match opstr {
        "EQU" => 0x00,
        "DC" => 0x01,
        "DS" => 0x02,
        _ => -1,
    }
}
fn match_reg(regstr: &str) -> i32 {
    match regstr {
        "R0" => 0,
        "R1" => 1,
        "R2" => 2,
        "R3" => 3,
        "R4" => 4,
        "R5" => 5,
        "R6" | "SP" => 6,
        "R7" | "FP" => 7,
        _ => -1,
    }
}

fn parse_number(numstr: &str) -> Option<i32> {
    let minus = numstr.starts_with('-');
    let mut numstr2: String;
    if minus {
        numstr2 = numstr.chars().into_iter().skip(1).collect();
        println!("{}", numstr2)
    } else {
        numstr2 = numstr.into();
    }
    let prefix: String = numstr2.chars().into_iter().take(2).collect();
    let value: i32;
    let radix;
    match prefix.as_str() {
        // u32 and then cast to i32 because from_str_radix doesn't seem to understand two's complement
        "0B" => radix = 2,
        "0O" => radix = 8,
        "0X" => radix = 16,
        _ => radix = 10,
    }
    if radix != 10 {
        numstr2 = numstr2.chars().skip(2).collect();
    }
    match u32::from_str_radix(numstr2.as_str(), radix) {
        Ok(int) => value = int as i32,
        Err(_) => return None,
    }

    if minus {
        return Some(-value);
    }
    Some(value)
}
