use std::collections::HashMap;
use num_traits::ToPrimitive;


const FORBIDDEN_CHARS: [char; 6] = [
    '(', ')', // parentheses
    '@', '=', // mode signs
    '-', // minus
    ':', // what was semicolon used for, again?
];

#[derive(PartialEq)]
enum Keyword {
    Directive,
    Constant,
    Data,
    Code,
    Register,
    None,
}

#[derive(Copy, Clone)]
enum Instruction {
    // Standard
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
    SVC = 0x70,

    // Extended
    IEXIT = 0x39,
    HLT = 0x71,
    HCF = 0x72,
}

pub struct Compiler {
    pub output: String,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler {
            output: "".into(),
        }
    }
}

impl Compiler {
    pub fn compile(&mut self, source: String) -> Result<String, String> {

        // Start address. Zero if none.
        let mut org: Option<usize> = None;

        // Dictionary of const names and their valuers.
        let const_symbols: HashMap<String, i16>;

        // Dictionaries of labels and their offsets in their respective segments.
        let data_symbols: HashMap<String, usize>;
        let code_symbols: HashMap<String, usize>;

        // These contain source processed into integers.
        let data_segment: Vec<i32>;
        let code_segment: Vec<i32>;

        // Source code distilled into "Statement" structs.
        let mut statements;
        match get_statements(&source) {
            Ok(val) => statements = val,
            Err(e) => return Err(e)
        }

        // Guard: Multiple definition
        match check_multiple_definition(&statements) {
            Ok(()) => {}
            Err(e) => return Err(e)
        }

        // Get directives
        for statement in &statements {
            if statement.statement_type != Keyword::Directive {
                continue;
            }

            let keyword_string = statement.words[0].to_uppercase();
            let keyword = keyword_string.as_str();
            let line = statement.line;

            // Guard: Label
            if !statement.label.is_none() {
                return Err(format!("You can't label a compiler directive! '{}' on line {}", keyword, line));
            }

            match keyword {
                "ORG" => {
                    let value;

                    // Guard: Origin already defined
                    if org != None {
                        return Err(format!("Found 'ORG' on line {}, but it's already defined!", line));
                    }

                    // Guard: Incorrect number of words
                    match statement.words.len() {
                        2 => (), // expected amount
                        1 => return Err(format!("No value given for '{}' on line {}", keyword, line)),
                        _ => return Err(format!("Too many words for '{}' on line {}", keyword, line)),
                    }

                    // Get value
                    match parse_number(&statement.words[1]) {
                        Ok(val) => value = val,
                        Err(e) => return Err(format!("Can't parse value on line {}: {}", line, e))
                    }

                    // Guard: Value out of range
                    if value < 0 {
                        return Err(format!("You tried to offset the program to a negative address! '{}' on line {}", keyword, line));
                    }

                    // Ok.
                    org = Some(value as usize);
                }
                _ => {
                    return Err(format!("Compiler made an error on line {}: {} is not a directive.", statement.line, keyword));
                }
            }
        }

        // Get constants
        match parse_const_statements(&statements) {
            Ok(result) => const_symbols = result,
            Err(e) => return Err(e)
        }

        // Get variables
        match parse_data_statements(&mut statements, org) {
            Ok((seg, symbols)) => {
                data_segment = seg;
                data_symbols = symbols;
            }
            Err(e) => return Err(e)
        }

        // Get code
        code_symbols = get_code_labels(&statements);
        match parse_code_statements(&mut statements, org, &const_symbols, &data_symbols, &code_symbols) {
            Ok(seg) => {
                code_segment = seg;
            }
            Err(e) => return Err(e)
        }

        let binary;
        match build_b91(
            code_segment,
            data_segment,
            code_symbols,
            data_symbols,
            org,
        ) {
            Ok(result) => binary = result,
            Err(e) => return Err(e)
        }
        Ok(binary)
    }
}

/// This will Find all relevant source code lines, and break them into "Statements"
fn get_statements(source: &String) -> Result<Vec<Statement>, String> {
    let mut statements: Vec<Statement> = Vec::new();

    for (i, text) in source.lines().enumerate() {
        let mut text = text.to_owned();

        let statement_type: Keyword;
        let line = i + 1;
        let label: Option<String>;
        let comment: Option<String>;

        // Get comment and remove it from the text line
        match text.split_once(';') {
            Some((before, after)) => {
                comment = Some(after.to_string());
                text = before.to_owned();
            }
            None => comment = None,
        }

        // Split the text line into words
        text = text.replace(",", " ");
        let mut words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
        if words.is_empty() {
            continue;
        }

        // Get label and remove it from keywords
        if get_keyword_type(&words[0]) == Keyword::None {
            label = Some(words[0].to_owned());
            words.remove(0);
        } else {
            label = None
        }

        // Find the statement's type by looking at the first word.
        let keyword_string = words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        match get_keyword_type(keyword) {
            Keyword::None => {
                return Err(format!("Unknown keyword '{}' on line {}\n{}", keyword, line, text));
            }
            Keyword::Register => {
                return Err(format!("Unexpected register '{}' on line {}\n{}", keyword, line, text));
            }
            Keyword::Directive => statement_type = Keyword::Directive,
            Keyword::Data => statement_type = Keyword::Data,
            Keyword::Constant => statement_type = Keyword::Constant,
            Keyword::Code => statement_type = Keyword::Code,
        }

        // Make a statement.
        statements.push(Statement {
            statement_type,
            words,
            line: i + 1,
            label,
            comment,
        })
    }
    return Ok(statements);
}

/// This will simply check if the same label exists in multiple statements.
fn check_multiple_definition(statements: &Vec<Statement>) -> Result<(), String> {
    let mut failed = false;
    let mut definitions: HashMap<String, Vec<usize>> = HashMap::new();

    // Collect all definitions
    for statement in statements {
        if let Some(label) = &statement.label {
            if definitions.contains_key(label) {
                // Defined already! mark failed add this line to the entry.
                failed = true;
                definitions.get_mut(label).unwrap().push(statement.line);
            } else {
                // First definition: create an entry that contains this line.
                let mut vec: Vec<usize> = Vec::new();
                vec.push(statement.line);
                definitions.insert(label.clone(), vec);
            }
        }
    }

    // Failure: Iterate through all definitions and construct an error message
    if failed {
        let mut err_mgs = "Multiple definitions:".to_string();
        for (label, lines) in definitions {
            if lines.len() == 1 {
                continue;
            }
            err_mgs += format!("\n    {} on lines: {:?}", label, lines).as_str()
        }
        return Err(err_mgs);
    }

    // Didn't fail
    Ok(())
}

/// This will create a dictionary of all constants (keyword EQU).
/// Note: Do check for multiple definitions _before_ this.
fn parse_const_statements(statements: &Vec<Statement>) -> Result<HashMap<String, i16>, String> {
    let mut consts: HashMap<String, i16> = HashMap::new();
    for statement in statements {
        if statement.statement_type != Keyword::Constant {
            continue;
        }
        let keyword_string = statement.words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        let line = statement.line;
        let value;

        // Guard: Keyword sanity check
        if keyword != "EQU" {
            return Err(format!("Line {}: '{}' is not 'EQU'. This is compiler's fault, not yours. Please file an issue.", line, keyword));
        }

        // Guard: No label
        if statement.label.is_none() {
            return Err(format!("Constants require a name! '{}' on line {}", keyword, line));
        }
        let label = statement.label.clone().unwrap();

        // Guard: Incorrect number of words
        match statement.words.len() {
            2 => (), // expected amount
            1 => return Err(format!("No value given for '{}' on line {}", keyword, line)),
            _ => return Err(format!("Too many words for '{}' on line {}", keyword, line)),
        }

        // Get value
        match parse_number(&statement.words[1]) {
            Ok(val) => value = val,
            Err(e) => return Err(format!("Error parsing value on line {}: {}", line, e))
        }

        // Guard: Value out of range
        if value < i16::MIN as i32 {
            return Err(format!("Value out of range on line {}. Got {}, but minimum is {}. Note that constants are 16-bit only.", line, value, i16::MIN));
        } else if value > i16::MAX as i32 {
            return Err(format!("Value out of range on line {}. Got {}, but maximum is {}. Note that constants are 16-bit only.", line, value, i16::MAX));
        }

        // Done
        consts.insert(label, value as i16);
    }
    return Ok(consts);
}

/// Creates data segment and data symbols
fn parse_data_statements(
    statements: &mut Vec<Statement>, org: Option<usize>)
    -> Result<(Vec<i32>, HashMap<String, usize>), String>
{
    let mut data_segment = Vec::new();
    let mut data_symbols = HashMap::new();

    for statement in statements {
        if statement.statement_type != Keyword::Data {
            continue;
        }

        let org = org.unwrap_or(0);
        let keyword_string = statement.words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        let line = statement.line;
        let value;

        // Guard: Word count
        match statement.words.len() {
            2 => (), // expected amount
            1 => return Err(format!("No value given for '{}' on line {}", keyword, line)),
            _ => return Err(format!("Too many words for '{}' on line {}", keyword, line)),
        }

        // Get value
        match parse_number(&statement.words[1]) {
            Ok(val) => value = val,
            Err(e) => return Err(format!("Error parsing value on line {}: {}", line, e))
        }

        match keyword {
            // Data Constant - store a value
            "DC" => {
                // Push data
                data_segment.push(value);

                // Add symbol, if labeled
                if let Some(label) = &statement.label {
                    data_symbols.insert(label.clone(), org + data_segment.len());
                }
            }
            // Data Segment - allocate space
            "DS" => {
                // Guard: out of range
                if value < 0 {
                    return Err(format!("You tried to allocate a negative number of addresses! '{}' on line {}", keyword, line));
                } else if value == 0 {
                    return Err(format!("You tried to allocate a zero addresses! '{}' on line {}", keyword, line));
                }

                // Push data
                for _ in 0..value {
                    data_segment.push(0);
                }

                // Add symbol, if labeled
                if let Some(label) = &statement.label {
                    data_symbols.insert(label.clone(), org + data_segment.len());
                }
            }
            _ => return Err(format!("Error: '{}' on line {} is not a variable keyword. This is compiler's fault, not yours. Please file an issue.", keyword, line)),
        }
    }
    Ok((data_segment, data_symbols))
}

/// Before actually parsing the code, we need to know possible code labels the code might reference.
fn get_code_labels(statements: &Vec<Statement>) -> HashMap<String, usize> {
    let mut code_symbols = HashMap::new();

    let mut code_offset = 0;
    for statement in statements {
        if statement.statement_type != Keyword::Code {
            continue;
        }
        code_offset += 1;

        // Add symbol, if labeled
        if let Some(label) = &statement.label {
            code_symbols.insert(label.clone(), code_offset);
        }
    }
    code_symbols
}

/// Creates code segment and code symbols
fn parse_code_statements(
    statements: &mut Vec<Statement>,
    org: Option<usize>,
    const_symbols: &HashMap<String, i16>,
    data_symbols: &HashMap<String, usize>,
    code_symbols: &HashMap<String, usize>,
) -> Result<Vec<i32>, String>
{
    let mut code_segment = Vec::new();

    for statement in statements {
        if statement.statement_type != Keyword::Code {
            continue;
        }

        let org = org.unwrap_or(0);
        let mut words = statement.words.clone();
        let keyword_string = statement.words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        // Remove oper keyword
        words.remove(0);
        let line = statement.line;
        let mut value: i32;

        // indirect memory access.
        // Normally addressing mode goes like this:
        // "=" => 0,
        // " " => 1,
        // "@" => 2
        // with some instructions indirect addressing is disabled
        // "=" is not allowed,
        // " " => 0,
        // "@" => 1

        // Operands. Most instructions use the defaults, but not all.
        let mut op1: String = "".to_string();
        let mut op2: String = "".to_string();

        if words.len() >= 1 {
            op1 = words[0].clone();
        }
        if words.len() >= 2 {
            op2 = words[1].clone();
        }

        let opcode: i32;
        let ri: i32;
        let mut mode: i32 = 1;
        let rj: i32;
        let addr: i32;

        match keyword {
            "NOP" => {
                assert_op_count(words.len(), 0, line)?;
                opcode = Instruction::NOP as i32;
            }
            "STORE" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::STORE as i32;
                mode -= 1; // no indirect
            }
            "LOAD" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::LOAD as i32;
            }
            "IN" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::IN as i32;
            }
            "OUT" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::OUT as i32;
            }
            "ADD" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::OUT as i32;
            }
            "SUB" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SUB as i32;
            }
            "MUL" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::MUL as i32;
            }
            "DIV" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::DIV as i32;
            }
            "MOD" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::MOD as i32;
            }
            "AND" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::AND as i32;
            }
            "OR" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::OR as i32;
            }
            "XOR" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::XOR as i32;
            }
            "SHL" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SHL as i32;
            }
            "SHR" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SHR as i32;
            }
            "NOT" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::NOT as i32;
            }
            "SHRA" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SHRA as i32;
            }
            "COMP" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::COMP as i32;
            }
            "JUMP" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JUMP as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JNEG" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JNEG as i32;
                mode -= 1; // no indirect
            }
            "JZER" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JZER as i32;
                mode -= 1; // no indirect
            }
            "JPOS" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JPOS as i32;
                mode -= 1; // no indirect
            }
            "JNNEG" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JNNEG as i32;
                mode -= 1; // no indirect
            }
            "JNZER" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JNZER as i32;
                mode -= 1; // no indirect
            }
            "JNPOS" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JNPOS as i32;
                mode -= 1; // no indirect
            }
            "JLES" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JLES as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JEQU" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JEQU as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JGRE" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JGRE as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JNLES" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JNLES as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JNEQU" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JNEQU as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JNGRE" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JNGRE as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "CALL" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::CALL as i32;
                mode -= 1; // no indirect
            }
            "EXIT" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::EXIT as i32;
            }
            "PUSH" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::PUSH as i32;
            }
            "POP" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::POP as i32;
            }
            "PUSHR" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::PUSHR as i32;
            }
            "POPR" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::POPR as i32;
            }
            "SVC" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SVC as i32;
            }
            // Extended
            "IEXIT" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::IEXIT as i32;
            }
            "HLT" => {
                assert_op_count(words.len(), 0, line)?;
                opcode = Instruction::HLT as i32;
            }
            "HCF" => {
                assert_op_count(words.len(), 0, line)?;
                opcode = Instruction::HCF as i32;
            }
            _ => return Err(format!("Compiler error: {} is not an instruction. Please file an issue.", keyword)),
        }

        op1 = op1.to_uppercase();

        // Parse op1: Rj
        if op1.is_empty() {
            rj = 0;
        } else if let Ok(val) = get_reg(op1.as_str()) {
            rj = val
        } else {
            return Err(format!("Line {}: Invalid register '{}' for first operand!", line, op1));
        }

        // Parse op2: Ri, mode, addr
        if op2.is_empty() {
            ri = 0;
            addr = 0;
        } else if let Ok(parsed) = parse_op2(op2.as_str()) {
            // Mode
            mode += parsed.mode;
            // Register
            match get_reg(parsed.reg.as_str()) {
                Ok(val) => ri = val,
                Err(_) => return Err(format!("Line {}: Invalid register '{}' for second operand!", line, op2)),
            }
            // Address (is empty)
            if parsed.addr.as_str() == "" {
                mode -= 1;
                addr = 0;
            }
            // Address (is builtin const)
            else if let Ok(val) = get_builtin_const(&parsed.addr) {
                addr = val;
            }
            // Address (is const)
            else if let Some(val) = const_symbols.get(&parsed.addr) {
                addr = val.to_i32().unwrap();
            }
            // Address (is variable)
            else if let Some(offset) = data_symbols.get(&parsed.addr) {
                addr = (org + offset).to_i32().unwrap();
            }
            // Address (is code)
            else if let Some(offset) = code_symbols.get(&parsed.addr) {
                addr = (org + offset).to_i32().unwrap();
            }
            // Address (is number)
            else if let Ok(val) = parse_number(parsed.addr.as_str()) {
                addr = val;
            }
            // Address (is invalid)
            else {
                return Err(format!("Line {}: invalid address: {}", line, parsed.addr));
            }
        } else {
            return Err(format!("Line {}: Couldn't parse second operand: {}", line, op2));
        }

        value = opcode << 24;
        value += rj << 21;
        value += mode << 19;
        value += ri << 16;
        match i16::try_from(addr) {
            Ok(val) => value += (val as i32) & 0xffff,
            Err(_) => {
                match u16::try_from(addr) {
                    Ok(val) => value += (val as i32) & 0xffff,
                    Err(_) => return Err(format!("Compiler error: can't fit addr '{}' into 16 bits on line {}", addr, line))
                }
            }
        }

        code_segment.push(value);
    }
    if code_segment.is_empty() {
        return Err("No code found!".into());
    }
    Ok(code_segment)
}

/// This is a shortcut to make the assertion a oneliner in parse_code_statements.
fn assert_op_count(n: usize, m: usize, ln: usize) -> Result<(), String> {
    if n != m {
        return Err(format!("Line {}: Too many terms!", ln));
    }
    Ok(())
}

fn build_b91(
    code_segment: Vec<i32>,
    data_segment: Vec<i32>,
    code_symbols: HashMap<String, usize>,
    data_symbols: HashMap<String, usize>,
    org: Option<usize>,
) -> Result<String, String>
{
    let org = org.unwrap_or(0);
    let code_size = code_segment.len();
    let fp_start = org + code_size - 1;
    let data_start = code_size + org;
    let sp_start = fp_start + data_segment.len();

    let mut return_str = "___b91___\n".to_string();

    // --- Code segment
    return_str += "___code___\n";
    // Code start and FP
    return_str += format!("{} {}\n", org.to_string(), fp_start.to_string()).as_str();
    // Actual code
    for i in code_segment {
        return_str += format!("{}\n", i.to_string()).as_str();
    }

    // --- Data segment
    return_str += "___data___\n";
    // Data start and SP
    return_str += format!("{} {}\n", data_start.to_string(), sp_start.to_string()).as_str();
    // Actual data
    for i in data_segment {
        return_str += format!("{}\n", i.to_string()).as_str();
    }

    // --- Symbol table
    return_str += "___symboltable___\n";
    // Variables:
    for (label, offset) in data_symbols {
        let addr = data_start + offset;
        return_str += format!("{} {}\n", label, addr).as_str();
    }
    // Code labels
    for (label, offset) in code_symbols {
        let addr = org + offset;
        return_str += format!("{} {}\n", label, addr).as_str();
    }

    // --- End
    return_str += "___end___\n";


    println!("{}", return_str);
    Ok(return_str)
}

fn get_keyword_type(keyword: &str) -> Keyword {
    let keyword_string = keyword.to_uppercase();
    let keyword = keyword_string.as_str();

    if let Ok(_) = get_instruction(keyword) {
        return Keyword::Code;
    }
    if let Ok(_) = get_reg(keyword) {
        return Keyword::Register;
    }
    if keyword == "EQU" {
        return Keyword::Constant;
    }
    if keyword == "DS" || keyword == "DC" {
        return Keyword::Data;
    }
    if keyword == "ORG" {
        return Keyword::Directive;
    }
    Keyword::None
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

fn get_instruction(keyword: &str) -> Result<i32, String> {
    let value;
    match keyword {
        "NOP" => value = Instruction::NOP,
        "STORE" => value = Instruction::STORE,
        "LOAD" => value = Instruction::LOAD,
        "IN" => value = Instruction::IN,
        "OUT" => value = Instruction::OUT,
        "ADD" => value = Instruction::ADD,
        "SUB" => value = Instruction::SUB,
        "MUL" => value = Instruction::MUL,
        "DIV" => value = Instruction::DIV,
        "MOD" => value = Instruction::MOD,
        "AND" => value = Instruction::AND,
        "OR" => value = Instruction::OR,
        "XOR" => value = Instruction::XOR,
        "SHL" => value = Instruction::SHL,
        "SHR" => value = Instruction::SHR,
        "NOT" => value = Instruction::NOT,
        "SHRA" => value = Instruction::SHRA,
        "COMP" => value = Instruction::COMP,
        "JUMP" => value = Instruction::JUMP,
        "JNEG" => value = Instruction::JNEG,
        "JZER" => value = Instruction::JZER,
        "JPOS" => value = Instruction::JPOS,
        "JNNEG" => value = Instruction::JNNEG,
        "JNZER" => value = Instruction::JNZER,
        "JNPOS" => value = Instruction::JNPOS,
        "JLES" => value = Instruction::JLES,
        "JEQU" => value = Instruction::JEQU,
        "JGRE" => value = Instruction::JGRE,
        "JNLES" => value = Instruction::JNLES,
        "JNEQU" => value = Instruction::JNEQU,
        "JNGRE" => value = Instruction::JNGRE,
        "CALL" => value = Instruction::CALL,
        "EXIT" => value = Instruction::EXIT,
        "PUSH" => value = Instruction::PUSH,
        "POP" => value = Instruction::POP,
        "PUSHR" => value = Instruction::PUSHR,
        "POPR" => value = Instruction::POPR,
        "SVC" => value = Instruction::SVC,

        "IEXIT" => value = Instruction::IEXIT,
        "HLT" => value = Instruction::HLT,
        "HCF" => value = Instruction::HCF,
        _ => return Err(format!("{} is not an instruction.", keyword)),
    }
    Ok(value as i32)
}

fn get_builtin_const(sym: &str) -> Result<i32, String> {
    match sym {
        "SHRT_MAX" => Ok(32767),
        "SHRT_MIN" => Ok(-32768),
        //"USHRT_MAX" => Ok(65535),
        "INT_MAX" => Ok(2147483647),
        "INT_MIN" => Ok(-2147483648),
        //"UINT_MAX" => Ok(-1),

        "CRT" => Ok(0),
        "KBD" => Ok(1),
        "RTC" => Ok(2),
        "HALT" => Ok(11),
        "READ" => Ok(12),
        "WRITE" => Ok(13),
        "TIME" => Ok(14),
        "DATE" => Ok(15),
        _ => Err(format!("{} is not a builtin symbol.", sym)),
    }
}

fn parse_number(numstr: &str) -> Result<i32, String> {
    let mut num_string: String = numstr.into();

    // Catch sign and remove it from string
    let minus = numstr.starts_with('-');
    if minus {
        num_string = numstr.chars().into_iter().skip(1).collect();
    }

    let prefix: String = num_string.chars().into_iter().take(2).collect();
    let value: i32;
    let radix;
    match prefix.as_str() {
        // u32 and then cast to i32 because from_str_radix doesn't seem to understand two's complement
        "0B" | "0b" => radix = 2,
        "0O" | "0o" => radix = 8,
        "0X" | "0x" => radix = 16,
        _ => radix = 10,
    }
    if radix != 10 {
        num_string = num_string.chars().skip(2).collect();
    }
    match u32::from_str_radix(num_string.as_str(), radix) {
        Ok(int) => value = int as i32,
        Err(e) => return Err(format!("{}: '{}'", e, numstr)),
    }

    match minus {
        true => Ok(-value),
        false => Ok(value),
    }
}

struct Statement {
    pub statement_type: Keyword,
    pub words: Vec<String>,
    pub line: usize,
    pub label: Option<String>,
    pub comment: Option<String>,
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
                match addr.to_uppercase().as_str() {
                    // That was actually a reg
                    "R0" | "R1" | "R2" | "R3" | "R4" | "R5" | "R6" | "R7" | "SP" | "FP" => {
                        return Ok(Op2 {
                            // Ended nicely
                            mode,
                            addr: String::new(),
                            reg: addr.to_uppercase(),
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
                    _ => reg += c.to_uppercase().to_string().as_str(),
                },
                None => {
                    return Err("Parentheses not closed".into());
                }
            }
        }
    }
    Ok(Op2 { mode, addr, reg })
}
