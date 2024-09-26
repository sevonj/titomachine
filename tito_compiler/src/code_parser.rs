// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! TTKTK - TTK-91 ToolKit
//!
//! TiToMachine k91 assembler - Instruction parsing module.
//!
pub(crate) use crate::compiler::{str_to_builtin_const, str_to_integer, Statement, Symbol};
use crate::instructions::{OpCode, Register};
use std::collections::HashMap;
use std::str::FromStr;

pub fn parse_instruction(
    statement: Statement,
    symbol_table: &HashMap<String, Symbol>,
) -> Result<i32, String> {
    let mut words = statement.words.clone();
    let keyword_string = statement.words[0].to_uppercase();
    let keyword = keyword_string.as_str();

    // Remove oper keyword
    words.remove(0);
    let line = statement.line;

    // Get opcode
    let opcode;
    match OpCode::from_str(keyword) {
        Ok(op) => opcode = op,
        Err(e) => return Err(format!("Line {}: {}", line, e)),
    }

    // Assert correct number of operands
    if words.len() != opcode.get_operand_count() {
        return Err(format!(
            "Line {}: Invalid number of operands for {}. Expected {}, but got {}",
            line,
            keyword,
            opcode.get_operand_count(),
            words.len()
        ));
    }

    // Get operand words
    let op1: String;
    let op2: String;

    match words.len() {
        0 => {
            op1 = "R0".to_string();
            op2 = String::new();
        }
        1 => {
            // Some jumps use op2 but not op1. Swap them, if appropriate.
            if opcode.is_op2_only() {
                op1 = "R0".to_string();
                op2 = words[0].clone();
            } else {
                op1 = words[0].to_uppercase();
                op2 = String::new();
            }
        }
        2 => {
            op1 = words[0].to_uppercase();
            op2 = words[1].clone();
        }
        _ => panic!(
            "Line {}: wtf, word count is '{}'. '{:?}'",
            line,
            words.len(),
            words
        ),
    }

    // Get first register
    let rj;
    match Register::from_str(op1.as_str()) {
        Ok(register) => rj = register,
        Err(e) => return Err(format!("Line {}: {}", line, e)),
    }

    // Parse op2: Ri, mode, addr
    let mode;
    let ri;
    let addr: i32;

    if op2.is_empty() {
        mode = opcode.get_default_mode();
        ri = Register::R0;
        addr = 0;
    } else if let Ok(parsed) = parse_op2(op2.as_str()) {
        // Mode
        mode = opcode.get_default_mode() + parsed.mode;

        // Register
        ri = parsed.register;

        // Address
        if parsed.addr.as_str() == "" {
            // (is empty)
            addr = 0;
        } else if let Ok(val) = str_to_builtin_const(&parsed.addr) {
            // (is builtin const)
            addr = val;
        } else if let Some(symbol) = symbol_table.get(&parsed.addr) {
            // (is in symbol table)
            addr = symbol.offset
        } else if let Ok(val) = str_to_integer(parsed.addr.as_str()) {
            // (is number)
            addr = val;
        } else {
            return Err(format!("Line {}: invalid address: {}", line, parsed.addr));
        }
    } else {
        return Err(format!(
            "Line {}: Couldn't parse second operand: {}",
            line, op2
        ));
    }

    if mode < 0 && mode > 2 {
        return Err(format!("Line {}: Mode {} is out of range", line, mode));
    }
    if addr < i16::MIN as i32 && addr > u16::MAX as i32 {
        return Err(format!("Line {}: Address {} is out of range", line, addr));
    }

    let mut value;
    value = (opcode as i32) << 24;
    value += (rj as i32) << 21;
    value += mode << 19;
    value += (ri as i32) << 16;
    value += addr & 0xffff;
    Ok(value)
}

/// Used by parse_op2()
struct Op2 {
    pub mode: i32,
    pub addr: String,
    pub register: Register,
}

/// Parse second operand: "=123(R2)"
fn parse_op2(input_str: &str) -> Result<Op2, String> {
    let mut mode: i32 = 0;
    let mut addr = String::new();
    //let mut chars = input_str.chars();

    let mut text = input_str.to_string();

    // Catch mode sign
    if input_str.starts_with("=") {
        mode = -1;
        text.remove(0);
    } else if input_str.starts_with("@") {
        mode = 1;
        text.remove(0);
    }

    // Catch minus sign
    if input_str.starts_with("-") {
        addr += "-";
        text.remove(0);
    }

    // We're done already: Second operand text is a register with no address.
    if let Ok(register) = Register::from_str(text.as_str()) {
        // Do not allow negative direct register addressing "-R1"
        if addr.as_str() == "-" {
            return Err(format!("Negative direct register addressing '{}' is not allowed. The minus sign only affects address portion.", input_str));
        }

        return Ok(Op2 {
            mode: mode - 1, // Register only decrements because of direct reg addressing
            addr,
            register,
        });
    }

    let register;
    // Second operand _contains_ register in parentheses
    if let Some((before_open, after_open)) = text.split_once('(') {
        match after_open.split_once(')') {
            Some((register_string, after_close)) => {
                register = Register::from_str(register_string)?;

                // Err: There's stuff on both sides of the parentheses!
                if !before_open.is_empty() && !after_close.is_empty() {
                    return Err(format!("Failed to parse second operand: '{}'", input_str));
                }

                // Nothing outside parentheses; we're done
                if before_open.is_empty() && after_close.is_empty() {
                    return Ok(Op2 {
                        mode,
                        addr,
                        register,
                    });
                }

                // One side is empty and one is not.
                text = before_open.to_string() + after_close;
            }
            None => return Err("Unclosed parentheses".to_string()),
        }
    } else {
        register = Register::R0;
    }

    // _No register_ in second operand. It's just address.
    addr += text.as_str();
    Ok(Op2 {
        mode,
        addr,
        register,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Keyword;
    /*
    Addressing modes require some careful testing.

        Code                    Desired effect  Examples, explanation
        - "=" prefix            decrement mode
        - "@" prefix            increment mode
        - no address            decrement mode  "R1", "R0"
        - no addr, neg reg      illegal         "-R1", "-R0"    Titokone refuses to compile, thinking it's label.
        - @@0                   illegal         Would get mode 2 on store command. Is illegal and should be illegal.
        - ==0                   illegal         Would get decrement by 2 Is illegal and should be illegal.

        Other:
        - Reg in parentheses implies presence of address 0: "(R1)" == "0(R1)"
        - Mode sign must be the first character. "=-1" is OK. "-=1" should fail.

     */
    #[test]
    /// Go through all combinations of mode signs
    fn test_parse_op2_mode_sign() {
        // Very basic
        assert_eq!(parse_op2("=0").unwrap().mode, -1); // Immediate value
        assert_eq!(parse_op2("0").unwrap().mode, 0); // Direct memory
        assert_eq!(parse_op2("@0").unwrap().mode, 1); // Indirect memory

        assert_eq!(parse_op2("=1").unwrap().mode, -1); // Immediate value
        assert_eq!(parse_op2("1").unwrap().mode, 0); // Direct memory
        assert_eq!(parse_op2("@1").unwrap().mode, 1); // Indirect memory

        assert_eq!(parse_op2("=55555").unwrap().mode, -1); // Immediate value
        assert_eq!(parse_op2("55555").unwrap().mode, 0); // Direct memory
        assert_eq!(parse_op2("@55555").unwrap().mode, 1); // Indirect memory

        assert_eq!(parse_op2("=-55555").unwrap().mode, -1); // Immediate value
        assert_eq!(parse_op2("-55555").unwrap().mode, 0); // Direct memory
        assert_eq!(parse_op2("@-55555").unwrap().mode, 1); // Indirect memory

        assert_eq!(parse_op2("=0x500").unwrap().mode, -1); // Immediate value
        assert_eq!(parse_op2("0x500").unwrap().mode, 0); // Direct memory
        assert_eq!(parse_op2("@0x500").unwrap().mode, 1); // Indirect memory
    }

    #[test]
    /// Only the first character should count as a mode sign.
    fn test_parse_op2_mode_sign_first() {
        assert_eq!(parse_op2("=1").unwrap().mode, -1);
        assert_eq!(parse_op2("@1").unwrap().mode, 1);

        // The sign should not affect mode and should not be removed from the string.
        assert_eq!(parse_op2("-=1").unwrap().mode, 0);
        assert_eq!(parse_op2("-=1").unwrap().addr, "-=1");
        assert_eq!(parse_op2("-@1").unwrap().mode, 0);
        assert_eq!(parse_op2("-@1").unwrap().addr, "-@1");

        assert_eq!(parse_op2("0=1").unwrap().mode, 0);
        assert_eq!(parse_op2("0=1").unwrap().addr, "0=1");
        assert_eq!(parse_op2("0@1").unwrap().mode, 0);
        assert_eq!(parse_op2("0@1").unwrap().addr, "0@1");

        // First mode sign should count and be removed, but not the second
        assert_eq!(parse_op2("==1").unwrap().mode, -1);
        assert_eq!(parse_op2("==1").unwrap().addr, "=1");
        assert_eq!(parse_op2("@@1").unwrap().mode, 1);
        assert_eq!(parse_op2("@@1").unwrap().addr, "@1");
        assert_eq!(parse_op2("=@1").unwrap().mode, -1);
        assert_eq!(parse_op2("=@1").unwrap().addr, "@1");
        assert_eq!(parse_op2("@=1").unwrap().mode, 1);
        assert_eq!(parse_op2("@=1").unwrap().addr, "=1");
    }

    #[test]
    /// Test everything that should result in indexed addressing (mode 0)
    fn test_parse_op2_mode_direct_register() {
        // No address: should decrement mode.
        assert_eq!(parse_op2("R0").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("R1").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("R2").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("R3").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("R4").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("R5").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("R6").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("R7").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("SP").unwrap().mode, -1); // Direct register
        assert_eq!(parse_op2("FP").unwrap().mode, -1); // Direct register
    }

    #[test]
    /// Minus sign only affects address. These shouldn't pass the compiler.
    fn test_parse_op2_mode_direct_register_neg() {
        assert!(parse_op2("-R0").is_err());
        assert!(parse_op2("-R1").is_err());
        assert!(parse_op2("-FP").is_err());
    }

    #[test]
    /// Parentheses implies address: should not decrement mode.
    /// In practice, "(R1)" should be treated the same as "0(R1)"    fn test_parse_op2_mode_indexed_implied() {
    fn test_parse_op2_mode_indexed_implied() {
        assert_eq!(parse_op2("(R0)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("(R1)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("(R2)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("(SP)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("(FP)").unwrap().mode, 0); // Indexed addressing
    }

    #[test]
    fn test_parse_op2_mode_indexed() {
        assert_eq!(parse_op2("0(R3)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0(R4)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0(R5)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0(R6)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0(R7)").unwrap().mode, 0); // Indexed addressing

        assert_eq!(parse_op2("0x123(R0)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("-0x123(R1)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0b101(R2)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("-0b1010(R3)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0o1234(R4)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("-0o1234(R5)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0XaAbB(R6)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0O8887(R7)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("0B1010(SP)").unwrap().mode, 0); // Indexed addressing
        assert_eq!(parse_op2("9999(FP)").unwrap().mode, 0); // Indexed addressing
    }

    #[test]
    /// Indexed with mode sign
    fn test_parse_op2_mode_indexed_immediate() {
        assert_eq!(parse_op2("=(R0)").unwrap().mode, -1); // Indexed immediate
        assert_eq!(parse_op2("=(R1)").unwrap().mode, -1); // Indexed immediate
        assert_eq!(parse_op2("=(R2)").unwrap().mode, -1); // Indexed immediate
        assert_eq!(parse_op2("=0(R0)").unwrap().mode, -1); // Indexed immediate
        assert_eq!(parse_op2("=0(R1)").unwrap().mode, -1); // Indexed immediate
        assert_eq!(parse_op2("=0(R2)").unwrap().mode, -1); // Indexed immediate
        assert_eq!(parse_op2("=0x123(R0)").unwrap().mode, -1); // Indexed immediate
        assert_eq!(parse_op2("=-0x123(R1)").unwrap().mode, -1); // Indexed immediate
    }

    #[test]
    /// Indexed with mode sign
    fn test_parse_op2_mode_indexed_indirect() {
        assert_eq!(parse_op2("@(R7)").unwrap().mode, 1); // Indexed indirect
        assert_eq!(parse_op2("@(SP)").unwrap().mode, 1); // Indexed indirect
        assert_eq!(parse_op2("@(FP)").unwrap().mode, 1); // Indexed indirect
        assert_eq!(parse_op2("@0(R7)").unwrap().mode, 1); // Indexed indirect
        assert_eq!(parse_op2("@0(SP)").unwrap().mode, 1); // Indexed indirect
        assert_eq!(parse_op2("@0(FP)").unwrap().mode, 1); // Indexed indirect
        assert_eq!(parse_op2("@0b101(R2)").unwrap().mode, 1); // Indexed indirect
        assert_eq!(parse_op2("@-0b1010(R3)").unwrap().mode, 1); // Indexed indirect
    }

    #[test]
    fn test_parse_instruction() {
        // Dummy symbol table
        let map = Default::default();
        assert_eq!(
            parse_instruction(dummy_statement("add r1 =0"), &map).unwrap(),
            287309824
        );
        assert_eq!(
            parse_instruction(dummy_statement("add r1 @(r1)"), &map).unwrap(),
            288423936
        );
        assert_eq!(
            parse_instruction(dummy_statement("store r1 @0"), &map).unwrap(),
            19398656
        );
        assert_eq!(
            parse_instruction(dummy_statement("store r1 @(r1)"), &map).unwrap(),
            19464192
        );
    }

    fn dummy_statement(text: &str) -> Statement {
        Statement {
            statement_type: Keyword::Code,
            label: None,
            words: text.split_whitespace().map(str::to_string).collect(),
            line: 0,
            comment: None,
        }
    }
}
