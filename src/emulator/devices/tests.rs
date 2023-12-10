use std::{thread, time::Duration};

use crate::emulator::{cpu::CPU, devices::PMIO};

use super::{dev_pic::DevPIC, Bus};

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
