use std::collections::HashMap;

use num_traits::ToPrimitive;

const FORBIDDEN_CHARS: [char; 6] = [
    '(', ')', // parentheses
    '@', '=', // mode signs
    '-', // minus
    ':',
];

pub struct Compiler {
    pub output: String,
    temp_symbols_const: HashMap<String, i32>, // constants (EQU declarations)
    temp_symbols_code: HashMap<String, i32>,  // code segment (instruction labels)
    temp_symbols_data: HashMap<String, i32>,  // data segment (DS, DC declarations)
    symbol_table: HashMap<String, i32>,       // Final symbol table
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler {
            output: "".into(),
            temp_symbols_const: HashMap::new(),
            temp_symbols_code: HashMap::new(),
            temp_symbols_data: HashMap::new(),
            symbol_table: HashMap::new(),
        }
    }
}

impl Compiler {
    fn out(&mut self, text: String) {
        self.output += (text + "\n").as_str();
    }

    fn clear(&mut self) {
        self.output = "".into();
        self.temp_symbols_const = HashMap::new();
        self.temp_symbols_code = HashMap::new();
        self.temp_symbols_data = HashMap::new();
        self.symbol_table = HashMap::new();
    }

    fn is_symbol_valid(&mut self, symbol: String) -> Result<(), String> {
        if symbol.len() == 0 {
            return Err(format!("Symbol is empty. Compiler did something wrong.",));
        }
        // Existing symbols
        if let Ok(_) = get_reg(symbol.as_str()) {
            return Err(format!(
                "Cannot define \"{}\", it is a register!   ",
                symbol
            ));
        } else if let Ok(_) = get_instruction(symbol.as_str()) {
            return Err(format!(
                "Cannot define \"{}\", it is an instruction!",
                symbol
            ));
        } else if let Ok(_) = get_pseudoinstr(symbol.as_str()) {
            return Err(format!(
                "Cannot define \"{}\", it is a pseudoinstruction",
                symbol
            ));
        } else if let Ok(_) = get_builtin_symbol(symbol.as_str()) {
            return Err(format!(
                "Cannot define \"{}\", it is a builtin symbol.",
                symbol
            ));
        } else if let Ok(_) = get_builtin_const(symbol.as_str()) {
            return Err(format!(
                "Cannot define \"{}\", it is a builtin constant.",
                symbol
            ));
        } else if self.temp_symbols_const.contains_key(&symbol) {
            return Err(format!(
                "Cannot define \"{}\", it is already defined!",
                symbol
            ));
        } else if self.temp_symbols_code.contains_key(&symbol) {
            return Err(format!(
                "Cannot define \"{}\", it is already defined!",
                symbol
            ));
        } else if self.temp_symbols_data.contains_key(&symbol) {
            return Err(format!(
                "Cannot define \"{}\", it is already defined!",
                symbol
            ));
        }

        // Forbidden characters
        let mut chars = symbol.chars();
        if chars.nth(0).unwrap().is_numeric() {
            return Err(format!(
                "Cannot define \"{}\", first character cannot be numeric!",
                symbol
            ));
        }
        loop {
            match chars.next() {
                Some(c) => {
                    if FORBIDDEN_CHARS.contains(&c) {
                        return Err(format!(
                            "Symbol \"{}\" contains forbidden character: {}",
                            symbol, c
                        ));
                    }
                }
                None => break,
            }
        }
        Ok(())
    }

    pub fn compile(&mut self, source: String) -> Result<String, ()> {
        self.clear();

        let mut org: Option<i32> = None;
        let mut data: Vec<i32> = Vec::new();
        let mut prog: Vec<i32> = Vec::new();

        // Process the lines: remove anything unnecessary
        let mut processed_lines: Vec<String> = Vec::new();
        for line in source.lines() {
            let mut result_line = String::new();
            // empty line
            if line == "" {
                processed_lines.push("".into()); // push even empty lines to preserve ln no.
                continue;
            }
            result_line += line.split(";").take(1).next().unwrap(); // Remove comments
            result_line = result_line.replace(",", " "); // Commas to spaces
            let words: Vec<String> = result_line.split_whitespace().map(str::to_string).collect();
            result_line = words.join(" ");
            result_line = result_line.to_uppercase();
            processed_lines.push(result_line);
        }

        // Collect Symbols
        let mut ln = 0;
        let mut prog_size = 0;
        let mut data_size = 0;
        for line in &processed_lines {
            ln += 1;
            if line.len() == 0 {
                continue;
            }
            let mut words: Vec<String> = line.split_whitespace().map(str::to_string).collect();

            let symbol;
            if let Ok(()) = self.is_symbol_valid(words[0].clone()) {
                self.out(format!("Line {}: Found symbol \"{}\"", ln, words[0]));
                symbol = Some(words.remove(0));
                if words.len() == 0 {
                    self.out(format!(
                        "Line {}: There's no instruction after symbol {}!",
                        ln,
                        symbol.unwrap()
                    ));
                    return Err(());
                }
            } else {
                symbol = None;
            }

            // At this point, possible symbol has been removed from words.

            // Must not be anonymous
            if symbol == None && words[0].as_str() == "EQU" {}

            // Code
            if let Ok(_) = get_instruction(words[0].as_str()) {
                if let Some(sym) = symbol {
                    self.temp_symbols_code.insert(sym, prog_size); // Add label if any
                }
                prog_size += 1;
                continue;
            }
            if words.len() < 2 {
                self.out(format!("Line {}: No value entered for symbol!", ln));
                return Err(());
            }
            // Pseudoinstr
            match words[0].as_str() {
                "EQU" => {
                    if let Some(sym) = symbol {
                        match parse_number(words[1].as_str()) {
                            Ok(n) => {
                                self.temp_symbols_const.insert(sym, n);
                            }
                            Err(e) => {
                                self.out(format!("Line {}: {}", ln, e));
                                return Err(());
                            }
                        }
                    } else {
                        self.out(format!("Line {}: Constants cannot be anonymous!", ln));
                        return Err(());
                    }
                }
                "DC" => match parse_number(words[1].as_str()) {
                    Ok(n) => {
                        if let Some(sym) = symbol {
                            self.temp_symbols_data.insert(sym, data_size);
                        }
                        data_size += 1;
                        data.push(n);
                    }
                    Err(e) => {
                        self.out(format!("Line {}: {}", ln, e));
                        return Err(());
                    }
                },
                "DS" => match parse_number(words[1].as_str()) {
                    Ok(n) => {
                        if n <= 0 {
                            self.out(format!(
                                "Line {}: Segment size must be larger than zero!",
                                ln
                            ));
                            return Err(());
                        }
                        if let Some(sym) = symbol {
                            self.temp_symbols_data.insert(sym, data_size);
                        }
                        data_size += n;
                        for _ in 0..n {
                            data.push(0);
                        }
                    }
                    Err(e) => {
                        self.out(format!("Line {}: {}", ln, e));
                        return Err(());
                    }
                },
                "ORG" => {
                    if symbol != None {
                        self.out(format!("Line {}: Cannot name origin!", ln));
                        return Err(());
                    }
                    if org != None {
                        self.out(format!("Line {}: Origin redefined!", ln));
                        return Err(());
                    }
                    match parse_number(words[1].as_str()) {
                        Ok(n) => {
                            if n < 0 {
                                self.out(format!("Line {}: Origin must not be negative.", ln));
                                return Err(());
                            }
                            org = Some(n);
                        }
                        Err(e) => {
                            self.out(format!("Line {}: {}", ln, e));
                            return Err(());
                        }
                    }
                }
                _ => {
                    self.out(format!(
                        "Line {}: Compiler made an error :(\n(Pseudoinstruction matching)",
                        ln
                    ));
                    return Err(());
                }
            }
        }

        self.out("".into());
        self.out(format!("code size: {}", prog_size));
        self.out(format!("data size: {}", data_size));

        // Now we have collected all symbols and we also know the size of code and data sections.
        // It's let's add the declared symbols to the final symbol table
        let origin;
        match org {
            Some(o) => origin = o,
            None => origin = 0,
        }
        for entry in &self.temp_symbols_const {
            self.symbol_table
                .insert(entry.0.to_string(), entry.1 + origin);
        }
        for entry in &self.temp_symbols_code {
            self.symbol_table
                .insert(entry.0.to_string(), entry.1 + origin);
        }
        for entry in &self.temp_symbols_data {
            self.symbol_table
                .insert(entry.0.to_string(), entry.1 + prog_size + origin);
        }

        self.out("".into());
        self.out("Built symbol table.".into());
        self.out("".into());

        // Get Instructions
        let mut ln = 0;
        let mut off = 0;
        for line in &processed_lines {
            ln += 1;
            if line.len() == 0 {
                continue;
            }

            let mut words: Vec<String> = line.split_whitespace().map(str::to_string).collect();

            if let Err(_) = get_instruction(words[0].as_str()) {
                if let Ok(_) = get_instruction(words[1].as_str()) {
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
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x00;
                    op1 = "";
                    op2 = "";
                }
                "STORE" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x01;
                    op1 = &words[1];
                    op2 = &words[2];
                    mode -= 1; // no indirect
                }
                "LOAD" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x02;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "IN" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x03;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "OUT" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x04;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "ADD" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x11;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "SUB" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x12;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "MUL" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x13;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "DIV" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x14;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "MOD" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x15;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "AND" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x16;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "OR" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x17;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "XOR" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x18;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "SHL" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x19;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "SHR" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x1a;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "NOT" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x1b;
                    op1 = &words[1];
                    op2 = "";
                }
                "SHRA" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x1c;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "COMP" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x1f;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "JUMP" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x20;
                    op1 = "";
                    op2 = &words[1];
                    mode -= 1; // no indirect
                }
                "JNEG" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x21;
                    op1 = &words[1];
                    op2 = &words[2];
                    mode -= 1; // no indirect
                }
                "JZER" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x22;
                    op1 = &words[1];
                    op2 = &words[2];
                    mode -= 1; // no indirect
                }
                "JPOS" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x23;
                    op1 = &words[1];
                    op2 = &words[2];
                    mode -= 1; // no indirect
                }
                "JNNEG" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x24;
                    op1 = &words[1];
                    op2 = &words[2];
                    mode -= 1; // no indirect
                }
                "JNZER" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x25;
                    op1 = &words[1];
                    op2 = &words[2];
                    mode -= 1; // no indirect
                }
                "JNPOS" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x26;
                    op1 = &words[1];
                    op2 = &words[2];
                    mode -= 1; // no indirect
                }
                "JLES" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x27;
                    op1 = "";
                    op2 = &words[1];
                    mode -= 1; // no indirect
                }
                "JEQU" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x28;
                    op1 = "";
                    op2 = &words[1];
                    mode -= 1; // no indirect
                }
                "JGRE" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x29;
                    op1 = "";
                    op2 = &words[1];
                    mode -= 1; // no indirect
                }
                "JNLES" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x2a;
                    op1 = "";
                    op2 = &words[1];
                    mode -= 1; // no indirect
                }
                "JNEQU" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x2b;
                    op1 = "";
                    op2 = &words[1];
                    mode -= 1; // no indirect
                }
                "JNGRE" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x2c;
                    op1 = "";
                    op2 = &words[1];
                    mode -= 1; // no indirect
                }
                "CALL" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x31;
                    op1 = &words[1];
                    op2 = &words[2];
                    mode -= 1; // no indirect
                }
                "EXIT" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x32;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "PUSH" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x33;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "POP" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x34;
                    op1 = &words[1];
                    op2 = &words[2];
                    if let Err(_) = get_reg(op2) {
                        self.out(format!(
                            "Line {}: Second operand must be a register for POP",
                            ln
                        ));
                        return Err(());
                    }
                }
                "PUSHR" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x35;
                    op1 = &words[1];
                    op2 = "";
                }
                "POPR" => {
                    if words.len() != 2 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x36;

                    op1 = &words[1];
                    op2 = "0";
                }
                "SVC" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x70;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "IEXIT" => {
                    if words.len() != 3 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x39;
                    op1 = &words[1];
                    op2 = &words[2];
                }
                "HLT" => {
                    if words.len() != 1 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x71;
                    op1 = "";
                    op2 = "";
                }
                "HCF" => {
                    if words.len() != 1 {
                        self.out(format!("Line {}: Unacceptable amount of terms!", ln));
                        return Err(());
                    }
                    opcode = 0x72;
                    op1 = "";
                    op2 = "";
                }
                _ => {
                    self.out(format!(
                        "Line {}: Compiler made an error :(\n(Instruction matching)",
                        ln
                    ));
                    return Err(());
                }
            }

            if op1 == "" {
                rj = 0;
            } else {
                match get_reg(op1) {
                    Ok(n) => rj = n,
                    Err(_) => {
                        self.out(format!("Line {}: Invalid register!", ln));
                        return Err(());
                    }
                }
            }
            if op2 == "" {
                mode = 1;
                ri = 0;
                addr = 0;
            } else {
                // Mode
                match parse_op2(op2) {
                    Ok(parsed) => {
                        // Mode
                        mode += parsed.mode;

                        // Register
                        match parsed.reg.as_str() {
                            "R0" | "" => ri = 0,
                            "R1" => ri = 1,
                            "R2" => ri = 2,
                            "R3" => ri = 3,
                            "R4" => ri = 4,
                            "R5" => ri = 5,
                            "R6" | "SP" => ri = 6,
                            "R7" | "FP" => ri = 7,
                            _ => {
                                self.out(format!("Line {}: Invalid Register!", ln));
                                return Err(());
                            }
                        }

                        // Address is empty
                        if parsed.addr.as_str() == "" {
                            mode -= 1;
                            addr = 0;
                        }
                        // Address is in symbol table
                        else if self.symbol_table.contains_key(&parsed.addr) {
                            addr = self
                                .symbol_table
                                .get(&parsed.addr)
                                .unwrap()
                                .to_i32()
                                .unwrap();
                        } else {
                            match parse_number(parsed.addr.as_str()) {
                                // Address is a value
                                Ok(int) => addr = int,
                                Err(_) => {
                                    // Address is builtin const
                                    if let Ok(int) = get_builtin_const(parsed.addr.as_str()) {
                                        addr = int;
                                    }
                                    // Address is builtin symbol
                                    else if let Ok(int) = get_builtin_symbol(parsed.addr.as_str())
                                    {
                                        addr = int;
                                        self.symbol_table.insert(parsed.addr, int);
                                    }
                                    // Address is not okay
                                    else {
                                        self.out(format!(
                                            "Line {}: invalid address: \"{}\"",
                                            ln, parsed.addr
                                        ));
                                        return Err(());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.out(format!("Line {}: {}", ln, e));
                        return Err(());
                    }
                }
                if mode < 0 {
                    self.out(format!("Line {}: Invalid addressing mode!", ln));
                    return Err(());
                }
            }
            //self.out(format!("{}, {}, {}, {}, {}, ", opcode, rj, mode, ri, addr);
            //println!("opcode: {:2x}", opcode);
            if opcode == 0x27 {
                println!("jump: {}, {:b}", addr, addr)
            }

            let mut instruction: i32 = 0;
            instruction += opcode << 24;
            instruction += rj << 21;
            instruction += mode << 19;
            instruction += ri << 16;
            match i16::try_from(addr) {
                Ok(int) => {
                    instruction += (int as i32) & 0xffff;
                }
                Err(_) => match u16::try_from(addr) {
                    Ok(int) => {
                        instruction += (int as i32) & 0xffff;
                    }
                    Err(e) => {
                        self.out(format!("Line {}: {}", ln, e));
                        return Err(());
                    }
                },
            }

            prog.push(instruction);

            off += 1;
        }

        let prog_start = origin;
        let fp_start = prog_size - 1 + origin;
        let data_start = prog_size + origin;
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
        for (key, value) in &self.symbol_table {
            return_str += key.as_str();
            return_str += " ";
            return_str += value.to_string().as_str();
            return_str += "\n";
        }
        return_str += "___end___\n";

        self.out(format!("Compiled:\n{}", return_str));
        //println!("Compiled:\n{}", return_str);
        Ok(return_str)
    }
}

fn get_reg(regstr: &str) -> Result<i32, String> {
    match regstr {
        "R0" | "" => Ok(0),
        "R1" => Ok(1),
        "R2" => Ok(2),
        "R3" => Ok(3),
        "R4" => Ok(4),
        "R5" => Ok(5),
        "R6" | "SP" => Ok(6),
        "R7" | "FP" => Ok(7),
        _ => Err(format!("{} is not an instruction.", regstr)),
    }
}
fn get_instruction(opstr: &str) -> Result<i32, String> {
    match opstr {
        "NOP" => Ok(0x00),
        "STORE" => Ok(0x01),
        "LOAD" => Ok(0x02),
        "IN" => Ok(0x03),
        "OUT" => Ok(0x04),
        "ADD" => Ok(0x11),
        "SUB" => Ok(0x12),
        "MUL" => Ok(0x13),
        "DIV" => Ok(0x14),
        "MOD" => Ok(0x15),
        "AND" => Ok(0x16),
        "OR" => Ok(0x17),
        "XOR" => Ok(0x18),
        "SHL" => Ok(0x19),
        "SHR" => Ok(0x1A),
        "NOT" => Ok(0x1B),
        "SHRA" => Ok(0x1C),
        "COMP" => Ok(0x1F),
        "JUMP" => Ok(0x20),
        "JNEG" => Ok(0x21),
        "JZER" => Ok(0x22),
        "JPOS" => Ok(0x23),
        "JNNEG" => Ok(0x24),
        "JNZER" => Ok(0x25),
        "JNPOS" => Ok(0x26),
        "JLES" => Ok(0x27),
        "JEQU" => Ok(0x28),
        "JGRE" => Ok(0x29),
        "JNLES" => Ok(0x2A),
        "JNEQU" => Ok(0x2B),
        "JNGRE" => Ok(0x2C),
        "CALL" => Ok(0x31),
        "EXIT" => Ok(0x32),
        "PUSH" => Ok(0x33),
        "POP" => Ok(0x34),
        "PUSHR" => Ok(0x35),
        "POPR" => Ok(0x36),
        "SVC" => Ok(0x70),

        "IEXIT" => Ok(0x39),
        "HLT" => Ok(0x71),
        "HCF" => Ok(0x72),
        _ => Err(format!("{} is not an instruction.", opstr)),
    }
}
fn get_pseudoinstr(opstr: &str) -> Result<i32, String> {
    match opstr {
        "EQU" => Ok(0x00),
        "DC" => Ok(0x01),
        "DS" => Ok(0x02),
        "ORG" => Ok(0x03),
        _ => Err(format!("{} is not a pseudoinstruction.", opstr)),
    }
}

fn get_builtin_const(sym: &str) -> Result<i32, String> {
    match sym {
        "SHRT_MAX" => Ok(32767),
        "SHRT_MIN" => Ok(-32768),
        "USHRT_MAX" => Ok(65535),
        "INT_MAX" => Ok(2147483647),
        "INT_MIN" => Ok(-2147483648),
        "UINT_MAX" => Ok(-1),
        _ => Err(format!("{} is not a builtin_const.", sym)),
    }
}

fn get_builtin_symbol(sym: &str) -> Result<i32, String> {
    match sym {
        "CRT" => Ok(0),
        "KBD" => Ok(1),
        "RTC" => Ok(2),
        "HALT" => Ok(11),
        "READ" => Ok(12),
        "WRITE" => Ok(13),
        "TIME" => Ok(14),
        "DATE" => Ok(15),
        _ => Err(format!("{} is not a builtin_symbol.", sym)),
    }
}

fn parse_number(numstr: &str) -> Result<i32, String> {
    let minus = numstr.starts_with('-');
    let mut numstr2: String;
    if minus {
        numstr2 = numstr.chars().into_iter().skip(1).collect();
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
        Err(e) => return Err(e.to_string()),
    }

    match minus {
        true => Ok(-value),
        false => Ok(value),
    }
}

struct Op2 {
    pub mode: i32,
    pub addr: String,
    pub reg: String,
}

fn parse_op2(input_str: &str) -> Result<Op2, String> {
    let mode: i32;
    let mut addr = String::new();
    let mut reg = String::new();
    let mut chars = input_str.chars();

    // Collect "-@", etc.
    if input_str.starts_with("-=") {
        addr += "-";
        mode = -1;
        chars.next();
        chars.next();
    } else if input_str.starts_with("-@") {
        addr += "-";
        mode = 1;
        chars.next();
        chars.next();
    } else if input_str.starts_with("-") {
        addr += "-";
        mode = 0;
        chars.next();
    } else if input_str.starts_with("=") {
        mode = -1;
        chars.next();
    } else if input_str.starts_with("@") {
        mode = 1;
        chars.next();
    } else {
        mode = 0;
    }

    // Collect addr string
    let get_register;
    loop {
        match chars.next() {
            Some(c) => match c {
                // Next is register
                '(' => {
                    if addr.len() == 0 {
                        addr += "0";
                    }
                    get_register = true;
                    break;
                }
                // Collect char
                _ => addr += c.to_string().as_str(),
            },
            // String ended, no register.
            None => {
                get_register = false;
                match addr.as_str() {
                    // That was actually a reg
                    "R0" | "R1" | "R2" | "R3" | "R4" | "R5" | "R6" | "R7" | "SP" | "FP" => {
                        return Ok(Op2 {
                            // Ended nicely
                            mode,
                            addr: String::new(),
                            reg: addr,
                        });
                    }
                    // Addr was just addr
                    _ => {}
                }
                break;
            }
        }
    }

    // Collect Reg string
    if get_register {
        loop {
            match chars.next() {
                Some(c) => match c {
                    ')' => {
                        if chars.next() != None {
                            return Err("Text after register.".into());
                        }
                        break; // Ended nicely
                    }
                    // Collect char
                    _ => reg += c.to_string().as_str(),
                },
                None => {
                    return Err("Parentheses not closed".into());
                }
            }
        }
    }
    Ok(Op2 { mode, addr, reg })
}
