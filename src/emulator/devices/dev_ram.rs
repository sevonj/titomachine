//!
//! A simple RAM device.
//!
//! 0x2000 or 8192 addresses equals to 32KB.
//!
use super::{Device, MMIO};

/// A simple RAM device.
pub(crate) struct DevRAM {
    ram: Vec<i32>,
}

impl Default for DevRAM {
    fn default() -> Self {
        DevRAM {
            ram: vec![0; 0x2000],
        }
    }
}

impl Device for DevRAM {
    fn reset(&mut self) {
        self.ram = vec![0; 0x2000];
    }
    fn on(&mut self) {}
    fn off(&mut self) {}
}

impl MMIO for DevRAM {
    fn read(&mut self, addr: usize) -> Result<i32, ()> {
        if addr >= self.ram.len() {
            return Err(());
        }
        Ok(self.ram[addr])
    }
    fn write(&mut self, addr: usize, value: i32) -> Result<(), ()> {
        if addr >= self.ram.len() {
            return Err(());
        }
        self.ram[addr] = value;
        Ok(())
    }
}
