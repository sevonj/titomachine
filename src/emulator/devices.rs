//!
//! This module contains device traits, and the Bus struct, which parents all devices.
//!
//! A device means a piece of hardware that is made accessible to the program via IO calls.
//!
//! Every device is accessible from the Bus struct.
//!
//! If you're writing a new device, it must implement the Device trait, and also one of, or both MMIO and PMIO traits.

use self::{
    dev_crt::DevCRT, dev_display_classic::DevDisplayClassic, dev_kbd::DevKBD, dev_pic::DevPIC,
    dev_ram::DevRAM, dev_rtc::DevRTC, psg::DevPSG,
};
mod dev_crt;
mod dev_display_classic;
mod dev_kbd;
mod dev_pic;
mod dev_ram;
mod dev_rtc;
mod psg;
#[cfg(test)]
mod tests;

/// All devices should implement this trait.
pub(crate) trait Device {
    /// Completely reset the state of the device.
    fn reset(&mut self);
    /// Turns this device on.
    fn on(&mut self);
    /// Turns this device off (but might not fully clear its state).
    fn off(&mut self);
}

/// Memory Mapped IO: Any device that occupies memory addresses shall implement this trait.
pub(crate) trait MMIO {
    /// MMIO read. In implementation, address is **relative to device offset**, not global. So your first addr is always 0x0.
    fn read(&mut self, addr: usize) -> Result<i32, ()>;
    /// MMIO write. In implementation, address is **relative to device offset**, not global. So your first addr is always 0x0.
    fn write(&mut self, addr: usize, value: i32) -> Result<(), ()>;
}

/// Port Mapped IO: Any device that occupies ports shall implement this trait.
pub(crate) trait PMIO {
    /// PMIO read. In implementation, port index is **relative to device offset**, not global. So your first port is always 0x0.
    fn read_port(&mut self, port: u8) -> Result<i32, ()>;
    /// PMIO write. In implementation, port index is **relative to device offset**, not global. So your first port is always 0x0.
    fn write_port(&mut self, port: u8, value: i32) -> Result<(), ()>;
}

/// The Bus struct is the parent of all devices, and maps IO calls to them.
/// Essentially it determines the hardware configuration of the machine.
pub struct Bus {
    pub(crate) ram: DevRAM,
    pub(crate) display: DevDisplayClassic,
    pub(crate) crt: DevCRT,
    pub(crate) kbd: DevKBD,
    pub(crate) rtc: DevRTC,
    pub(crate) pic: DevPIC,
    pub(crate) psg: DevPSG,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: DevRAM::default(),
            display: DevDisplayClassic::default(),
            crt: DevCRT::default(),
            kbd: DevKBD::default(),
            rtc: DevRTC::default(),
            pic: DevPIC::default(),
            psg: DevPSG::default(),
        }
    }
    /// MMIO access
    pub(crate) fn read(&mut self, addr: u32) -> Result<i32, ()> {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1fff => self.ram.read(addr),
            0x2000..=0x6B00 => self.display.read(addr - 0x2000),
            _ => {
                println!("mem read fault: {:x}", addr);
                Err(())
            }
        }
    }
    pub(crate) fn write(&mut self, addr: u32, value: i32) -> Result<(), ()> {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1fff => self.ram.write(addr, value),
            0x2000..=0x6B00 => self.display.write(addr - 0x2000, value),
            _ => {
                println!("mem write fault: {:x}", addr);
                Err(())
            }
        }
    }
    /// PMIO access
    pub(crate) fn read_port(&mut self, port: i32) -> Result<i32, ()> {
        let port = port as usize;
        match port {
            0 => self.crt.read_port(0),
            1 => self.kbd.read_port(0),
            2 => self.rtc.read_port(0),
            //6 => stdin
            //7 => stdout
            0x20 => self.pic.read_port(0),
            0x21 => self.pic.read_port(1),
            0x22 => self.pic.read_port(2),
            _ => {
                println!("port read fault: {:x}", port);
                Err(())
            }
        }
    }
    pub(crate) fn write_port(&mut self, port: i32, value: i32) -> Result<(), ()> {
        let port = port as usize;
        match port {
            0 => self.crt.write_port(0, value),
            1 => self.kbd.write_port(0, value),
            2 => self.rtc.write_port(0, value),
            //6 => stdin
            //7 => stdout
            0x20 => self.pic.write_port(0, value),
            0x21 => self.pic.write_port(1, value),
            0x22 => self.pic.write_port(2, value),
            _ => {
                println!("port write fault: {:x}", port);
                Err(())
            }
        }
    }

    pub(crate) fn reset_devices(&mut self) {
        self.ram.reset();
        self.pic.reset();
        self.display.reset();
    }
}
