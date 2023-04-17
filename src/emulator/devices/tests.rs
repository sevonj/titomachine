use std::thread;

use crate::emulator::cpu::CPU;

use super::Bus;

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
