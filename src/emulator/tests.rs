use libttktk::compiler::compile;
use super::{cpu::CPU, devices::Bus, loader};

/// These tests depend on compiler and loader.
///
/// The tests run samples in a minimal emulator instance, and verifies that they run correctly.
///

#[test]
/// Tests load / store
fn test_cpu_mmio() {
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    bus.write(0, 0x02200037).unwrap(); // LOAD  R1, =55
    bus.write(1, 0x01200200).unwrap(); // STORE R1, 0x200
    bus.write(2, 0x02480200).unwrap(); // LOAD  R2, 0x200
    cpu.tick(&mut bus);
    cpu.tick(&mut bus);
    cpu.tick(&mut bus);
    let expected = 55;

    assert_eq!(cpu.debug_get_gpr(2), expected);
}

#[test]
fn test_cpu_arithmetic() {
    let prog = compile(include_str!("../../programs/tests/test_cpu_arithmetic.k91").into()).unwrap();
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.burn {
        cpu.tick(&mut bus)
    }
    let expected = 55;
    assert_eq!(cpu.debug_get_gprs()[2], expected) // The result is stored in R2.
}

#[test]
fn test_cpu_logical() {
    let prog = compile(include_str!("../../programs/tests/test_cpu_logical.k91").into()).unwrap();
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.burn {
        cpu.tick(&mut bus)
    }
    let expected = 55;
    assert_eq!(cpu.debug_get_gprs()[2], expected) // The result is stored in R2.
}

/// Exhaustive test of all jumps and conditions
#[test]
fn test_cpu_jumps() {
    let prog = compile(include_str!("../../programs/tests/test_cpu_jumps.k91").into()).unwrap();
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.burn {
        cpu.tick(&mut bus)
    }
    let expected = 55;
    assert_eq!(cpu.debug_get_gprs()[2], expected) // The result is stored in R2.
}

/// Execute a simple subroutine.
#[test]
fn test_cpu_subroutines() {
    let prog = compile(include_str!("../../programs/tests/test_cpu_subroutines.k91").into()).unwrap();
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.burn {
        cpu.tick(&mut bus)
    }
    let expected = 55;
    assert_eq!(cpu.debug_get_gprs()[2], expected) // The result is stored in R2.
}

/// Stack instructions
#[test]
fn test_cpu_stack() {
    let prog = compile(include_str!("../../programs/tests/test_cpu_stack.k91").into()).unwrap();
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.burn {
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
/// Test hlt hcf
#[test]
fn test_cpu_halt() {
    let mut cpu = CPU::new();
    let mut bus = Bus::new();

    assert!(!cpu.halt);
    assert!(!cpu.burn);

    bus.write(0, 0x71000000).unwrap(); // HLT
    cpu.tick(&mut bus);
    assert!(cpu.halt);
    assert!(!cpu.burn);

    cpu = CPU::new();

    bus.write(0, 0x72000000).unwrap(); // HCF
    cpu.tick(&mut bus);
    assert!(cpu.halt);
    assert!(cpu.burn);
}

/// Tests most exception types.
#[test]
fn test_cpu_exceptions() {
    let prog = compile(include_str!("../../programs/tests/test_cpu_exceptions.k91").into()).unwrap();
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    while !cpu.burn {
        cpu.tick(&mut bus)
    }
    let expected = 55;
    assert_eq!(cpu.debug_get_gprs()[2], expected) // The result is stored in R2.
}

/// Verify that IVT entries are loaded correctly
#[test]
fn test_loader_ivt() {
    let prog = compile(include_str!("../../programs/tests/test_loader_ivt.k91").into()).unwrap();
    let mut cpu = CPU::new();
    let mut bus = Bus::new();
    loader::load_program(&mut bus, &mut cpu, &prog);
    for i in 0..=15 {
        assert_eq!(cpu.debug_get_ivt(i), 0x1000 + i as i32)
    }
}