///
/// devices/pic.rs
///
/// Programmable Interrupt Controller. This allows devices to trigger interrupts. It also contains a simple builtin timer.
///
/// The PIC contains two 8-bit registers:
/// - Mask Register: Which interrupts can fire
/// - Flag Register: Currently firing interrupts
///
/// Size of the registers may be increased later.
///
///
/// Interrupt map (subject to change):
/// | Bit | Device |
/// | --- | ------ |
/// | 0   |        |
/// | 1   | Timer  |
/// | 2   |        |
/// | 3   |        |
/// | 4   |        |
/// | 5   |        |
/// | 6   |        |
/// | 7   |        |
///
/// Ports:
///  - Port 0: Command
///  - Port 1: Mask
///  - Port 2: Timer
///
/// Read Behaviour:
/// | Port | Effect                |
/// | ---- | --------------------- |
/// | 0    | Returns Flag Register |
/// | 1    | Returns Mask Register |
/// | 2    | Returns Timer value   |
///
/// Write Behaviour:
/// | Port | Value | Effect                                                                      |
/// | ---- | ----- | --------------------------------------------------------------------------- |
/// | 0    | 0     | Clear Flag Register                                                         |
/// | 0    | 1     | Disable PIC (doesn't affect Mask or Flag registers)                         |
/// | 0    | 2     | Enable PIC (doesn't affect Mask or Flag registers)                          |
/// | 1    | any   | Set Mask Register. If a bit is cleared, it is also cleared in Flag Register |
/// | 2    | any   | Set Timer Reload Value. Resets timer.                                       |
///
/// So far this only accounts for timer. TODO: ther rest of it.
///
use super::{Device, PMIO};
use std::time::Duration;

const DEFAULT_MASK: u8 = 0b_00000010;
const MASK_TIMER: u8 = 0b_01000000;

pub(crate) struct DevPIC {
    pub(crate) firing: bool,

    enabled: bool,
    mask: u8,
    pub(crate) flag: u8,

    timer: Duration,
    timer_reload: u32,
}

impl Default for DevPIC {
    fn default() -> Self {
        DevPIC {
            firing: false,

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
        self.firing = false;
        self.enabled = true;
        self.mask = DEFAULT_MASK;
        self.flag = 0x00;
        self.timer_reload = 0;
        self.reset_timer();
    }
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

    /// Emulator shall call this to update PIC status
    pub(crate) fn update_status(&mut self) {
        if !self.enabled {
            self.firing = false;
            return;
        }
        self.firing = (self.flag & self.mask) != 0;
    }

    fn reset_timer(&mut self) {
        self.timer = Duration::from_millis(self.timer_reload as u64);
    }
}
