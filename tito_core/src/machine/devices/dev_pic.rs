//!
//! Programmable Interrupt Controller.
//!
//! <div class="warning">
//! The PIC, and the interrupt system are very incomplete and need more rethinking.
//! </div>
//!
//! The PIC allows devices to trigger interrupts. It also contains a simple builtin timer.
//!
//! ```
//! The PIC contains two 8-bit registers:
//! - Mask Register: Which interrupts can fire
//! - Flag Register: Currently firing interrupts
//!
//! Size of the registers may be increased later.
//!
//!
//! Interrupt map (subject to change):
//! | Bit | Device |
//! | --- | ------ |
//! | 0   |        |
//! | 1   | Timer  |
//! | 2   |        |
//! | 3   |        |
//! | 4   |        |
//! | 5   |        |
//! | 6   |        |
//! | 7   |        |
//!
//! Ports:
//!  - Port 0: Command
//!  - Port 1: Mask
//!  - Port 2: Timer
//!
//! Read Behaviour:
//! | Port | Effect                |
//! | ---- | --------------------- |
//! | 0    | Returns Flag Register |
//! | 1    | Returns Mask Register |
//! | 2    | Returns Timer value   |
//!
//! Write Behaviour:
//! | Port | Value | Effect                                                                      |
//! | ---- | ----- | --------------------------------------------------------------------------- |
//! | 0    | 0     | Clear Flag Register                                                         |
//! | 0    | 1     | Disable PIC (doesn't affect Mask or Flag registers)                         |
//! | 0    | 2     | Enable PIC (doesn't affect Mask or Flag registers)                          |
//! | 1    | any   | Set Mask Register. If a bit is cleared, it is also cleared in Flag Register |
//! | 2    | any   | Set Timer Reload Value. Resets timer.                                       |
//!
//! So far this only accounts for timer. TODO: ther rest of it.
//!
use super::{Device, PMIO};
use std::time::Duration;

const DEFAULT_MASK: u8 = 0b_00000010;
const MASK_TIMER: u8 = 0b_00000010;

pub(crate) struct DevPIC {
    enabled: bool,
    mask: u8,
    pub(crate) flag: u8,

    timer: Duration,
    timer_reload: u32,
}

impl Default for DevPIC {
    fn default() -> Self {
        DevPIC {
            enabled: true,
            mask: DEFAULT_MASK,
            flag: 0x00,

            timer: Duration::ZERO,
            timer_reload: 0,
        }
    }
}

impl Device for DevPIC {
    fn reset(&mut self) {
        self.enabled = true;
        self.mask = DEFAULT_MASK;
        self.flag = 0x00;
        self.timer_reload = 0;
        self.reset_timer();
    }
    fn on(&mut self) {}
    fn off(&mut self) {}
}

impl PMIO for DevPIC {
    fn read_port(&mut self, port: u8) -> Result<i32, ()> {
        match port {
            0 => Ok(self.flag as i32),
            1 => Ok(self.mask as i32),
            2 => Ok(self.timer.as_millis() as i32),
            _ => Err(()),
        }
    }
    fn write_port(&mut self, port: u8, value: i32) -> Result<(), ()> {
        match port {
            0 => match value {
                0 => self.flag = 0,
                1 => self.enabled = false,
                2 => self.enabled = true,
                _ => (),
            },
            1 => self.mask = value as u8,
            2 => {
                self.timer_reload = value as u32;
                self.reset_timer()
            }
            _ => return Err(()),
        }
        Ok(())
    }
}

impl DevPIC {
    /// Emulator shall call this to advance the timer
    pub(crate) fn update_timer(&mut self, d: Duration) {
        if self.timer_reload == 0 {
            return;
        }
        match self.timer.checked_sub(d) {
            Some(newvalue) => self.timer = newvalue,
            None => {
                // Timer ran out
                self.reset_timer();
                if self.mask & MASK_TIMER != 0 {
                    self.flag |= MASK_TIMER;
                }
            }
        }
    }

    pub(crate) fn is_firing(&mut self) -> bool {
        (self.flag & self.mask) != 0 && self.enabled
    }

    fn reset_timer(&mut self) {
        self.timer = Duration::from_millis(self.timer_reload as u64);
    }
}

mod tests {
    use super::*;
    use std::time::Duration;

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

}