use std::{thread, time::Duration};

use crate::emulator::{cpu::CPU, devices::PMIO};

use super::{dev_pic::DevPIC, Bus};

/// This test has the CPU output to CRT. The test listens if CRT sends the value via the channel.
#[test]
fn test_dev_crt() -> Result<(), ()> {
    let mut cpu = CPU::new();
    let mut bus = Bus::new();

    let (tx, rx) = std::sync::mpsc::channel();
    bus.crt.connect(tx);

    bus.write(0, 0x02200037)?; // LOAD R1, =55
    bus.write(1, 0x04200000)?; // OUT  R1, =0
    cpu.tick(&mut bus);
    cpu.tick(&mut bus);

    if rx.try_recv().unwrap() == 55 {
        return Ok(());
    }
    Err(())
}

///Test KBD. The test listens for input request, sends a value and checks if it gets loaded.
#[test]
fn test_dev_kbd() -> Result<(), ()> {
    let mut cpu = CPU::new();
    let mut bus = Bus::new();

    let (tx, rx) = std::sync::mpsc::channel();
    let (tx_req, rx_req) = std::sync::mpsc::channel();

    bus.kbd.connect(rx, tx_req);
    bus.write(0, 0x03400001)?; // IN R2, =1

    // Because KBD device locks the program, we have to put it into another thread.
    let cpu_thread = thread::spawn(move || {
        cpu.tick(&mut bus);
        cpu.debug_get_gpr(2)
    });

    // Send reply to the input request.
    // Because of thread timing, we don't check if kbd has requested for input.
    tx.send(55).unwrap();

    if cpu_thread.join().unwrap() == 55 {
        // Check if kbd ever sent the request.
        if let Err(_) = rx_req.try_recv() {
            return Err(());
        }
        return Ok(());
    }
    Err(())
}

/// PIC timer test.
#[test]
fn test_dev_pic_timer() -> Result<(), ()> {
    let mut pic = DevPIC::default();

    pic.write_port(2, 50)?; // Set timer to 50 ms
    assert!(!pic.is_firing());
    pic.update_timer(Duration::from_millis(49)); // Should not fire at 49 ms
    assert!(!pic.is_firing());
    pic.update_timer(Duration::from_millis(2)); // Should fire at 51 ms
    assert!(pic.is_firing());
    assert_eq!(pic.read_port(0)?, 0b_00000010); // It should be the timer bit that is set
    Ok(())
}

/// PIC generic test. Tests mask, flag register.
#[test]
fn test_dev_pic() -> Result<(), ()> {
    let mut pic = DevPIC::default();

    assert!(!pic.is_firing()); // Should not fire immediately
    pic.flag |= 0b_00000100; // Set framebuffer interrupt
    assert!(!pic.is_firing()); // Should not fire because the mask doesn't have framebuffer bit
    pic.write_port(1, 0b_00000100)?; // Set framebuffer bit
    assert!(pic.is_firing()); // It now should fire
    assert_eq!(pic.read_port(0)?, 0b_00000100); // check that correct bit is set
    Ok(())
}
