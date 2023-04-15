use super::super::editor::compiler::Compiler;
use super::{cpu::CPU, devices::Bus, loader};

/// These tests depend on compiler and loader.
///
/// The tests run samples in a minimal emulator instance, and verifies that they run correctly.
///

#[test]
fn test_cpu_arithmetic() {
    let prog = compile(include_str!("../../programs/tests/test_arithmetic.k91").into());
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.halt {
        cpu.tick(&mut bus)
    }
    let expected = 55;
    assert_eq!(cpu.debug_get_gprs()[2], expected) // The result is stored in R2.
}

#[test]
fn test_cpu_logical() {
    let prog = compile(include_str!("../../programs/tests/test_cpu_logical.k91").into());
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.halt {
        cpu.tick(&mut bus)
    }
    let expected = 55;
    assert_eq!(cpu.debug_get_gprs()[2], expected) // The result is stored in R2.
}

#[test]
fn test_cpu_jumps() {
    let prog = compile(include_str!("../../programs/tests/test_jumps.k91").into());
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.halt {
        cpu.tick(&mut bus)
    }
    let expected = 55;
    assert_eq!(cpu.debug_get_gprs()[2], expected) // The result is stored in R2.
}

#[test]
fn test_cpu_stack() {
    let prog = compile(include_str!("../../programs/tests/test_stack.k91").into());
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.halt {
        cpu.tick(&mut bus)
    }
    let expected_r0 = 100;
    let expected_r1 = 101;
    let expected_r2 = 102;
    let expected_r3 = 103;
    let expected_r4 = 104;
    let expected_r5 = 105;
    let regs = cpu.debug_get_gprs();
    assert_eq!(regs[0], expected_r0);
    assert_eq!(regs[1], expected_r1);
    assert_eq!(regs[2], expected_r2);
    assert_eq!(regs[3], expected_r3);
    assert_eq!(regs[4], expected_r4);
    assert_eq!(regs[5], expected_r5);
}

fn compile(source: String) -> String {
    let mut compiler = Compiler::default();
    compiler.compile(source).unwrap()
}
