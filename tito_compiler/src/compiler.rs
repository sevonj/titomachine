// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! TTKTK - TTK-91 ToolKit
//!
//! TiToMachine k91 assembly compiler.
//!

use crate::{
    code_parser::parse_instruction,
    instructions::{OpCode, Register},
};
use std::collections::HashMap;
use std::str::FromStr;

#[allow(dead_code)] // TODO: Not checked for anymore. Should be checked for symbol names.
const FORBIDDEN_CHARS: [char; 6] = [
    '(', ')', // parentheses
    '@', '=', // mode signs
    '-', // minus
    ':', // what was colon used for, again?
];

#[derive(PartialEq)]
pub(crate) enum Keyword {
    Directive,
    Const,
    Data,
    Code,
    Register,
    None,
}

#[derive(PartialEq, Debug)]
pub(crate) enum SymbolType {
    Const,
    Code,
    Data,
}

pub(crate) struct Symbol {
    pub offset: i32,
    pub symbol_type: SymbolType,
}

/// One of the first things that happens to a line of code is to be organized into this struct.
/// Statement holds the code as Vec<String>, and knows some high-level information and metadata
/// about it.
pub(crate) struct Statement {
    pub statement_type: Keyword,
    pub label: Option<String>,
    //
    pub words: Vec<String>,
    // Remaining keywords after label
    pub line: usize,
    #[allow(dead_code)] // Comments will be added to the output, eventually.
    pub comment: Option<String>,
}

pub fn compile(source: String) -> Result<String, String> {
    // Start address. Zero if none.
    let mut org: Option<usize> = None;

    // Dictionary of symbols
    let mut symbol_table: HashMap<String, Symbol>;

    // These contain source processed into integers.
    let data_segment: Vec<i32>;
    let mut code_segment: Vec<i32> = Vec::new();

    // Source code distilled into "Statement" structs.
    let mut statements;
    match code_to_statements(&source) {
        Ok(val) => statements = val,
        Err(e) => return Err(e),
    }

    // Guard: Multiple definition
    match assert_no_multiple_definition(&statements) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    // Get directives
    for statement in &statements {
        if statement.statement_type != Keyword::Directive {
            continue;
        }
        let keyword = statement.words[0].as_str();
        match keyword {
            "ORG" => {
                if org != None {
                    return Err(format!(
                        "Found 'ORG' on line {}, but it's already defined!",
                        statement.line
                    ));
                }
                org = Some(parse_org_directive(statement)?);
            }
            _ => {
                return Err(format!(
                    "Compiler made an error on line {}: {} is not a directive.",
                    statement.line, keyword
                ))
            }
        }
    }
    // Unpack org from option.
    let org = org.unwrap_or(0);

    // Create symbol table
    match create_symbol_table(&statements) {
        Ok(result) => symbol_table = result,
        Err(e) => return Err(e),
    }

    // Apply offsets to symbol table
    let code_size = get_code_segment_size(&statements);
    symbol_table = create_absolute_symbol_table(symbol_table, org + 0, org + code_size);

    // Get Data Segment
    match parse_data_statements(&mut statements) {
        Ok(segment) => data_segment = segment,
        Err(e) => return Err(e),
    }

    // Get Code Segment
    for statement in statements {
        if statement.statement_type == Keyword::Code {
            code_segment.push(parse_instruction(statement, &symbol_table)?);
        }
    }

    // Mash them together
    let binary;
    match build_b91(code_segment, data_segment, symbol_table, org) {
        Ok(result) => binary = result,
        Err(e) => return Err(e),
    }
    Ok(binary)
}

/// This will find all relevant source code lines, and break them into "Statements"
fn code_to_statements(source: &String) -> Result<Vec<Statement>, String> {
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
        if str_to_keyword_type(&words[0]) == Keyword::None {
            label = Some(words[0].to_owned());
            words.remove(0);
        } else {
            label = None
        }
        if words.is_empty() {
            return Err(format!("Unexpected end, {}\n{}", line, text));
        }

        // Find the statement's type by looking at the first word.
        let keyword_string = words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        match str_to_keyword_type(keyword) {
            Keyword::None => {
                return Err(format!(
                    "Unknown keyword '{}' on line {}\n{}",
                    keyword, line, text
                ));
            }
            Keyword::Register => {
                return Err(format!(
                    "Unexpected register '{}' on line {}\n{}",
                    keyword, line, text
                ));
            }
            Keyword::Directive => statement_type = Keyword::Directive,
            Keyword::Data => statement_type = Keyword::Data,
            Keyword::Const => statement_type = Keyword::Const,
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

fn parse_org_directive(statement: &Statement) -> Result<usize, String> {
    let value;
    let keyword_string = statement.words[0].to_uppercase();
    let keyword = keyword_string.as_str();
    let line = statement.line;

    // Guard: Label
    if !statement.label.is_none() {
        return Err(format!(
            "You can't label a compiler directive! '{}' on line {}",
            keyword, line
        ));
    }

    // Guard: Incorrect number of words
    match statement.words.len() {
        2 => (), // expected amount
        1 => return Err(format!("No value given for '{}' on line {}", keyword, line)),
        _ => return Err(format!("Too many words for '{}' on line {}", keyword, line)),
    }

    // Get value
    match str_to_integer(&statement.words[1]) {
        Ok(val) => value = val,
        Err(e) => return Err(format!("Can't parse value on line {}: {}", line, e)),
    }

    // Guard: Value out of range
    if value < 0 {
        return Err(format!(
            "You tried to offset the program to a negative address! '{}' on line {}",
            keyword, line
        ));
    }

    // Ok.
    Ok(value as usize)
}

fn create_symbol_table(statements: &Vec<Statement>) -> Result<HashMap<String, Symbol>, String> {
    let mut map = HashMap::new();
    let mut code_offset = -1;
    let mut data_offset = -1;
    for statement in statements {
        match statement.statement_type {
            Keyword::Const => {
                if statement.label.is_none() {
                    return Err(format!(
                        "Line {}: Constant requires a name!",
                        statement.line
                    ));
                }
            }
            Keyword::Code => code_offset += 1,
            Keyword::Data => data_offset += 1,
            _ => continue,
        }

        // Add symbol
        if let Some(label) = &statement.label {
            let symbol;
            match &statement.statement_type {
                Keyword::Const => {
                    symbol = Symbol {
                        offset: parse_const(statement)?,
                        symbol_type: SymbolType::Const,
                    }
                }
                Keyword::Code => {
                    symbol = Symbol {
                        offset: code_offset,
                        symbol_type: SymbolType::Code,
                    }
                }
                Keyword::Data => {
                    symbol = Symbol {
                        offset: data_offset,
                        symbol_type: SymbolType::Data,
                    }
                }
                _ => continue,
            }
            map.insert(label.clone(), symbol);
        }

        // Data segment: Compensate for remaining size.
        if statement.words[0].to_uppercase().as_str() == "DS" {
            if statement.words.len() < 2 {
                return Err(format!(
                    "Line {}: No size for data segment!",
                    statement.line
                ));
            }
            // -1 because we already incremented offset
            data_offset += str_to_integer(statement.words[1].as_str())? - 1
        }
    }
    Ok(map)
}

/// Apply relevant segment offsets to values.
fn create_absolute_symbol_table(
    relative_table: HashMap<String, Symbol>,
    code_start: usize,
    data_start: usize,
) -> HashMap<String, Symbol> {
    let mut absolute_table = HashMap::new();
    for (label, mut value) in &mut relative_table.into_iter() {
        match value.symbol_type {
            SymbolType::Const => {}
            SymbolType::Code => value.offset += code_start as i32,
            SymbolType::Data => value.offset += data_start as i32,
        }
        absolute_table.insert(label, value);
    }
    absolute_table
}

fn parse_const(statement: &Statement) -> Result<i32, String> {
    let keyword_string = statement.words[0].to_uppercase();
    let keyword = keyword_string.as_str();
    let line = statement.line;
    let value;

    match statement.words.len() {
        2 => (), // expected amount
        1 => return Err(format!("Line {}: No value given for '{}'", line, keyword)),
        _ => return Err(format!("Line {}: Too many words for '{}'", line, keyword)),
    }

    match str_to_integer(&statement.words[1]) {
        Ok(val) => value = val,
        Err(e) => return Err(format!("Line {}: Error parsing value: {}", e, line)),
    }

    if value < i16::MIN as i32 || value > i16::MAX as i32 {
        return Err(format!(
            "Line {}: Value out of range. Note that constants are 16-bit only.",
            line
        ));
    }
    Ok(value)
}

/// Creates data segment and data symbols
fn parse_data_statements(statements: &mut Vec<Statement>) -> Result<Vec<i32>, String> {
    let mut data_segment = Vec::new();

    for statement in statements {
        if statement.statement_type != Keyword::Data {
            continue;
        }

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
        match str_to_integer(&statement.words[1]) {
            Ok(val) => value = val,
            Err(e) => return Err(format!("Error parsing value on line {}: {}", line, e)),
        }

        match keyword {
            // Data Constant - store a value
            "DC" => data_segment.push(value),

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
            }
            _ => return Err(format!("Error: '{}' on line {} is not a variable keyword. This is compiler's fault, not yours. Please file an issue.", keyword, line)),
        }
    }
    Ok(data_segment)
}

fn get_code_segment_size(statements: &Vec<Statement>) -> usize {
    let mut code_offset = 0;
    for statement in statements {
        if statement.statement_type != Keyword::Code {
            continue;
        }
        code_offset += 1;
    }
    code_offset
}

/// Go through statements and check if same label comes up more than once.
fn assert_no_multiple_definition(statements: &Vec<Statement>) -> Result<(), String> {
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
    // Failure: Construct an error message
    if failed {
        let mut err_mgs = "Multiple definitions:".to_string();
        for (label, lines) in definitions {
            if lines.len() > 1 {
                err_mgs += format!("\n    {} on lines: {:?}", label, lines).as_str()
            }
        }
        return Err(err_mgs);
    }
    // Success
    Ok(())
}

fn build_b91(
    code_segment: Vec<i32>,
    data_segment: Vec<i32>,
    symbol_table: HashMap<String, Symbol>,
    org: usize,
) -> Result<String, String> {
    let code_size = code_segment.len();
    let fp_start: i32 = (org + code_size) as i32 - 1; // fp_start can be -1 if code_size == 0
    let data_start = code_size + org;
    let sp_start = fp_start + data_segment.len() as i32;

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
    for (label, value) in symbol_table.into_iter() {
        return_str += format!("{} {}\n", label, value.offset).as_str();
    }

    // --- End
    return_str += "___end___\n";

    Ok(return_str)
}

fn str_to_keyword_type(keyword: &str) -> Keyword {
    let keyword_string = keyword.to_uppercase();
    let keyword = keyword_string.as_str();

    if let Ok(_) = OpCode::from_str(keyword) {
        return Keyword::Code;
    }
    if let Ok(_) = Register::from_str(keyword) {
        return Keyword::Register;
    }
    if keyword == "EQU" {
        return Keyword::Const;
    }
    if keyword == "DS" || keyword == "DC" {
        return Keyword::Data;
    }
    if keyword == "ORG" {
        return Keyword::Directive;
    }
    Keyword::None
}

pub(crate) fn str_to_builtin_const(sym: &str) -> Result<i32, String> {
    match sym {
        "SHRT_MAX" => Ok(32767),
        "SHRT_MIN" => Ok(-32768),
        "USHRT_MAX" => Ok(65535),
        "INT_MAX" => Ok(2147483647),
        "INT_MIN" => Ok(-2147483648),
        "UINT_MAX" => Ok(-1),

        "CRT" => Ok(0),
        "KBD" => Ok(1),
        "RTC" => Ok(2),
        "HALT" => Ok(11),
        "READ" => Ok(12),
        "WRITE" => Ok(13),
        "TIME" => Ok(14),
        "DATE" => Ok(15),
        _ => Err(format!("{} is not a builtin constant.", sym)),
    }
}

pub(crate) fn str_to_integer(input_string: &str) -> Result<i32, String> {
    let mut num_string: String = input_string.to_lowercase();

    // Catch minus sign
    let minus = input_string.starts_with('-');
    if minus {
        num_string = input_string.chars().into_iter().skip(1).collect();
    }

    // Catch Bin/Oct/Hex prefix
    let prefix: String = num_string.chars().into_iter().take(2).collect();
    let radix = match prefix.as_str() {
        "0b" => 2,
        "0o" => 8,
        "0x" => 16,
        _ => 10,
    };
    if radix != 10 {
        num_string = num_string.chars().skip(2).collect();
    }

    // u32 and then cast to i32 because from_str_radix doesn't seem to understand two's complement
    let value: i32;
    match u32::from_str_radix(num_string.as_str(), radix) {
        Ok(int) => value = int as i32,
        Err(e) => return Err(format!("{}: '{}'", e, input_string)),
    }

    match minus {
        true => Ok(-value),
        false => Ok(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(str_to_integer("0b110100").unwrap(), 52);
        assert_eq!(str_to_integer("0o64").unwrap(), 52);
        assert_eq!(str_to_integer("52").unwrap(), 52);
        assert_eq!(str_to_integer("0x34").unwrap(), 52);

        assert_eq!(str_to_integer("-0b1").unwrap(), -1);
        assert_eq!(str_to_integer("-0o1").unwrap(), -1);
        assert_eq!(str_to_integer("-1").unwrap(), -1);
        assert_eq!(str_to_integer("-0x1").unwrap(), -1);

        // Treat as unsigned if no minus sign.
        assert_eq!(
            str_to_integer("0b11111111111111111111111111111111").unwrap(),
            -1
        );
        assert_eq!(str_to_integer("0o37777777777").unwrap(), -1);
        assert_eq!(str_to_integer("4294967295").unwrap(), -1);
        assert_eq!(str_to_integer("0xffffffff").unwrap(), -1);
    }

    #[test]
    fn test_parse_number_incorrect() {
        assert!(str_to_integer("0xabcdefg").is_err());
        assert!(str_to_integer("0o2345678").is_err());
        assert!(str_to_integer("0b012").is_err());
        assert!(str_to_integer("0b00a").is_err());

        assert!(str_to_integer("--0b1").is_err());
        assert!(str_to_integer("-=0o1").is_err());
        assert!(str_to_integer("-@1").is_err());
        assert!(str_to_integer("-0x0x1").is_err());
        assert!(str_to_integer("-0x0o1").is_err());
        assert!(str_to_integer("-x01").is_err());

        // Treat as unsigned if no minus sign.
        assert!(str_to_integer("0b111111111111111111111111111111111").is_err());
        assert!(str_to_integer("0o377777777777").is_err());
        assert!(str_to_integer("4294967296").is_err());
        assert!(str_to_integer("0xfffffffff").is_err());
    }

    #[test]
    fn test_parse_org_directive() {
        let statement = Statement {
            statement_type: Keyword::Directive,
            label: None,
            words: "ORG 50".split_whitespace().map(str::to_string).collect(),
            line: 0,
            comment: None,
        };
        assert_eq!(parse_org_directive(&statement).unwrap(), 50);

        let statement = Statement {
            statement_type: Keyword::Directive,
            label: None,
            words: "ORG 0x1000"
                .split_whitespace()
                .map(str::to_string)
                .collect(),
            line: 0,
            comment: None,
        };
        assert_eq!(parse_org_directive(&statement).unwrap(), 0x1000);
    }

    #[test]
    fn test_get_code_segment_size() {
        // Contains 6 instructions
        let source = "
        nop
        nop
        label dc 3
        const equ 00
        nop
        label2 nop
        add r1, =2
        ; nop
        load r1, =2
        ;
        ;"
        .to_string();
        let statements = code_to_statements(&source).unwrap();
        assert_eq!(get_code_segment_size(&statements), 6);
    }

    #[test]
    fn test_create_symbol_table() {
        let source = "
        const1 equ 1
        data1 dc 1
        data2 ds 2
        const2 equ 2
        ; nop
        nop
        ;
        code1 nop
        code2 add r1, =2
        data3 dc 3
        out r1, =3
        const3 equ 3
        code3 load r1, =2
        "
        .to_string();
        let statements = code_to_statements(&source).unwrap();
        let relative_table = create_symbol_table(&statements).unwrap();

        assert_eq!(relative_table.get("const1".into()).unwrap().offset, 1);
        assert_eq!(relative_table.get("const2".into()).unwrap().offset, 2);
        assert_eq!(relative_table.get("const3".into()).unwrap().offset, 3);

        // Note that data2 is a 2-address long segment
        assert_eq!(relative_table.get("data1".into()).unwrap().offset, 0);
        assert_eq!(relative_table.get("data2".into()).unwrap().offset, 1);
        assert_eq!(relative_table.get("data3".into()).unwrap().offset, 3);

        assert_eq!(relative_table.get("code1".into()).unwrap().offset, 1);
        assert_eq!(relative_table.get("code2".into()).unwrap().offset, 2);
        assert_eq!(relative_table.get("code3".into()).unwrap().offset, 4);
    }

    #[test]
    fn test_create_symbol_table_works_with_anonymous_vars() {
        let source = "
        data1 dc 0  ; 0
        dc 33       ; 1
        data2 dc 2  ; 2
        ds 3        ; 3-5
        data3 dc 6  ; 6
        "
        .to_string();
        let statements = code_to_statements(&source).unwrap();
        let relative_table = create_symbol_table(&statements).unwrap();

        assert_eq!(relative_table.get("data1".into()).unwrap().offset, 0);
        assert_eq!(relative_table.get("data2".into()).unwrap().offset, 2);
        assert_eq!(relative_table.get("data3".into()).unwrap().offset, 6);
    }

    #[test]
    fn test_create_symbol_table_correct_types() {
        let source = "
        const1 equ 1
        data1 dc 1
        code1 nop
        const2 equ 2
        data2 dc 2
        code2 nop
        "
        .to_string();
        let statements = code_to_statements(&source).unwrap();
        let relative_table = create_symbol_table(&statements).unwrap();

        assert_eq!(
            relative_table.get("const1".into()).unwrap().symbol_type,
            SymbolType::Const
        );
        assert_eq!(
            relative_table.get("const2".into()).unwrap().symbol_type,
            SymbolType::Const
        );
        assert_eq!(
            relative_table.get("data1".into()).unwrap().symbol_type,
            SymbolType::Data
        );
        assert_eq!(
            relative_table.get("data2".into()).unwrap().symbol_type,
            SymbolType::Data
        );
        assert_eq!(
            relative_table.get("code1".into()).unwrap().symbol_type,
            SymbolType::Code
        );
        assert_eq!(
            relative_table.get("code2".into()).unwrap().symbol_type,
            SymbolType::Code
        );
    }

    #[test]
    /// Make sure that code and data segment offsets are applied to symbol table correctly.
    fn test_create_absolute_symbol_table() {
        let code_start = 10;
        let data_start = 20;
        let mut relative_table = HashMap::new();

        relative_table.insert(
            "const".into(),
            Symbol {
                offset: 2,
                symbol_type: SymbolType::Const,
            },
        );
        relative_table.insert(
            "code".into(),
            Symbol {
                offset: 2,
                symbol_type: SymbolType::Code,
            },
        );
        relative_table.insert(
            "data".into(),
            Symbol {
                offset: 2,
                symbol_type: SymbolType::Data,
            },
        );

        let absolute_table = create_absolute_symbol_table(relative_table, code_start, data_start);

        assert_eq!(absolute_table.get("const".into()).unwrap().offset, 2);
        assert_eq!(absolute_table.get("code".into()).unwrap().offset, 12);
        assert_eq!(absolute_table.get("data".into()).unwrap().offset, 22);
    }

    #[test]
    fn test_build_b91_correct_symbol_values() {
        let mut symbol_table = HashMap::new();

        symbol_table.insert(
            "const".into(),
            Symbol {
                offset: 12,
                symbol_type: SymbolType::Const,
            },
        );
        symbol_table.insert(
            "code".into(),
            Symbol {
                offset: 34,
                symbol_type: SymbolType::Code,
            },
        );
        symbol_table.insert(
            "data".into(),
            Symbol {
                offset: 56,
                symbol_type: SymbolType::Data,
            },
        );

        // Org is set to an arbitrary nonzero value to make sure it doesn't affect label offsets anymore.
        let b91 = build_b91(Vec::new(), Vec::new(), symbol_table, 420).unwrap();
        let mut lines = b91.lines();

        // Skip until symboltable
        loop {
            if lines.next().unwrap() == "___symboltable___" {
                break;
            }
        }

        // Also make sure they're all found.
        let mut const_found = false;
        let mut code_found = false;
        let mut data_found = false;

        for _ in 0..3 {
            let words: Vec<String> = lines
                .next()
                .unwrap()
                .split_whitespace()
                .map(str::to_string)
                .collect();
            match words[0].as_str() {
                "const" => {
                    assert!(!const_found);
                    assert_eq!(words[1].as_str(), "12");
                    const_found = true
                }
                "code" => {
                    assert!(!code_found);
                    assert_eq!(words[1].as_str(), "34");
                    code_found = true
                }
                "data" => {
                    assert!(!data_found);
                    assert_eq!(words[1].as_str(), "56");
                    data_found = true
                }
                _ => panic!("wtf"),
            }
        }
    }

    #[test]
    fn test_label_no_instruction() {
        let source = "
        not_a_keyword  ;
        "
        .to_string();
        assert!(code_to_statements(&source).is_err());
    }

    #[test]
    fn test_cannot_redefine_const() {}

    #[test]
    fn test_cannot_redefine_var() {}

    #[test]
    fn test_cannot_redefine_code() {}

    #[test]
    /// Every combination of redefining a keyword with another type
    fn test_cannot_redefine_mixed() {}

    #[test]
    fn test_cannot_redefine_builtin_const() {}
}
