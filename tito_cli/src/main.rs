// SPDX-FileCopyrightText: 2024 sevonj
// SPDX-License-Identifier: MPL-2.0

//! Tito Unattended: A CLI-titomachine with a minimal hardware configuration.
//! Intended for automated testing.

use std::{fs, process::ExitCode, str::FromStr};

use clap::Parser;
use libttktk::disassembler::disassemble_instruction;
use tito_core::b91::B91;
use tito_core::machine::Machine;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Load a program from a .k91 source file.
    #[arg(short, long)]
    source: Option<String>,

    /// Load a program from .b91 binary file.
    #[arg(short, long)]
    binary: Option<String>,

    /// Validate the program against predetermined input and output (KBD, CRT).
    #[arg(short, long)]
    check: Option<String>,

    /// Store the log to a file.
    #[arg(short, long)]
    log: Option<String>,
}

/// Open, parse, and compile a K91 file.
fn read_k91(path: &str) -> Result<B91, ()> {
    let source = fs::read_to_string(path).expect("Could not to open the source file.");

    let compile_result = libttktk::compiler::compile(source);
    if let Err(e) = compile_result {
        println!("{}", e);
        return Err(());
    }
    let b91_result = B91::from_str(&compile_result.unwrap());
    if let Ok(b91) = b91_result {
        return Ok(b91);
    }
    Err(())
}

/// Open and parse a B91 file
fn read_b91(path: &str) -> Result<B91, ()> {
    let content = fs::read_to_string(path).expect("Could not to open the binary file.");
    let result = B91::from_str(&content);
    if let Ok(b91) = result {
        return Ok(b91);
    }
    Err(())
}

fn main() -> ExitCode {
    let args = Args::parse();

    // Can't have neither.
    if args.source.is_none() && args.binary.is_none() {
        println!("You must provide either a source file or a binary file.");
        return ExitCode::FAILURE;
    }

    // Can't have both.
    if args.source.is_some() && args.binary.is_some() {
        println!("You must not provide both a source file and a binary file.");
        return ExitCode::FAILURE;
    }

    // Get the program
    let b91_result = if args.source.is_some() {
        read_k91(&args.source.unwrap())
    } else {
        read_b91(&args.binary.unwrap())
    };
    if b91_result.is_err() {
        println!("Loading program failed.");
        return ExitCode::FAILURE;
    };
    let b91 = b91_result.unwrap();

    // Load and run the machine
    let mut machine = Machine::new();
    machine.load_b91(b91);

    loop {
        if machine.debug_get_hcf() {
            println!("HCF");
            break;
        }

        let pc = machine.debug_get_cpu_pc();
        let ins_str = if let Ok(ins) = machine.debug_read_mem(pc as u32) {
            disassemble_instruction(ins)
        } else {
            "Unable to read address at PC!".to_owned()
        };
        println!("{:#10x} - {}", pc, ins_str);

        machine.tick();
    }

    println!("made it!");

    ExitCode::SUCCESS
}
