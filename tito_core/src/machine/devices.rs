//!
//! This module contains device traits, and the Bus struct, which parents all devices.
//!
//! A device here means a piece of hardware that is made accessible to the program via IO instructions.
//!
//! If you're writing a new device, it must implement the Device trait, and at least one of the IO traits.

use self::{
    dev_crt::DevCRT,
    dev_display_classic::DevDisplayClassic,
    dev_kbd::DevKBD,
    // dev_pic::DevPIC,
    dev_ram::DevRAM,
    dev_rtc::DevRTC,
};

pub mod dev_crt;
pub mod dev_display_classic;
pub mod dev_kbd;
// mod dev_pic;
pub mod dev_ram;
pub mod dev_rtc;

/// All devices should implement this trait.
pub trait Device {
    /// Completely reset the state of the device.
    fn reset(&mut self);
    /// Turns this device on.
    fn on(&mut self);
    /// Turns this device off.
    fn off(&mut self);
    /// Pausing stops time for this device.
    fn set_pause(&mut self, paused: bool);
}

/// Memory Mapped IO: Any device that occupies memory addresses shall implement this trait.
pub trait MMIO: Device {
    /// MMIO read. In implementation, address is **relative to device offset**, not global. So your first addr is always 0x0.
    fn read(&mut self, addr: usize) -> Result<i32, ()>;
    /// MMIO write. In implementation, address is **relative to device offset**, not global. So your first addr is always 0x0.
    fn write(&mut self, addr: usize, value: i32) -> Result<(), ()>;
}

/// Port Mapped IO: Any device that occupies ports shall implement this trait.
pub trait PMIO: Device {
    /// PMIO read. In implementation, port index is **relative to device offset**, not global. So your first port is always 0x0.
    fn read_port(&mut self, port: u8) -> Result<i32, ()>;
    /// PMIO write. In implementation, port index is **relative to device offset**, not global. So your first port is always 0x0.
    fn write_port(&mut self, port: u8, value: i32) -> Result<(), ()>;
}

/// The Bus struct is the parent of all devices, and maps IO calls to them.
/// Essentially it determines the hardware configuration of the machine.
pub struct Bus {
    pub crt: DevCRT,
    pub display: DevDisplayClassic,
    pub kbd: DevKBD,
    // pub pic: DevPIC,
    pub ram: DevRAM,
    pub rtc: DevRTC,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            crt: DevCRT::default(),
            display: DevDisplayClassic::default(),
            kbd: DevKBD::default(),
            // pic: DevPIC::default(),
            ram: DevRAM::default(),
            rtc: DevRTC::default(),
        }
    }
    /// MMIO access
    pub fn read(&mut self, addr: u32) -> Result<i32, ()> {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1fff => self.ram.read(addr),
            0x2000..=0x6aff => self.display.read(addr - 0x2000),
            _ => {
                println!("mem read fault: 0x{:x}", addr);
                Err(())
            }
        }
    }
    pub fn write(&mut self, addr: u32, value: i32) -> Result<(), ()> {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1fff => self.ram.write(addr, value),
            0x2000..=0x6aff => self.display.write(addr - 0x2000, value),
            _ => {
                println!("mem write fault: 0x{:x}", addr);
                Err(())
            }
        }
    }
    /// PMIO access
    pub fn read_port(&mut self, port: i32) -> Result<i32, ()> {
        let port = port as usize;
        match port {
            0 => self.crt.read_port(0),
            1 => self.kbd.read_port(0),
            2 => self.rtc.read_port(0),
            //6 => stdin
            //7 => stdout
            //0x20 => self.pic.read_port(0),
            //0x21 => self.pic.read_port(1),
            //0x22 => self.pic.read_port(2),
            _ => {
                println!("port read fault: {:x}", port);
                Err(())
            }
        }
    }
    pub fn write_port(&mut self, port: i32, value: i32) -> Result<(), ()> {
        let port = port as usize;
        match port {
            0 => self.crt.write_port(0, value),
            1 => self.kbd.write_port(0, value),
            2 => self.rtc.write_port(0, value),
            //6 => stdin
            //7 => stdout
            //0x20 => self.pic.write_port(0, value),
            //0x21 => self.pic.write_port(1, value),
            //0x22 => self.pic.write_port(2, value),
            _ => {
                println!("port write fault: {:x}", port);
                Err(())
            }
        }
    }

    /*
     *
     *  TODO: What the this was the plan with all these:
     *
     */

    /// Clear all state
    pub fn reset(&mut self) {
        self.crt.reset();
        self.display.reset();
        self.kbd.reset();
        //self.pic.reset();
        self.ram.reset();
        self.rtc.reset();
    }

    /// Turn the device on. May affect state, not suitable for "pausing" the device.
    pub fn turn_on(&mut self) {
        self.crt.on();
        self.display.on();
        self.kbd.on();
        //self.pic.on();
        self.ram.on();
        self.rtc.on();
    }

    /// Turn the device off. May affect state, not suitable for "pausing" the device.
    pub fn turn_off(&mut self) {
        self.crt.off();
        self.display.off();
        self.kbd.off();
        //self.pic.off();
        self.ram.off();
        self.rtc.off();
    }

    pub fn set_pause(&mut self, paused: bool) {
        self.crt.set_pause(paused);
        self.display.set_pause(paused);
        self.kbd.set_pause(paused);
        //self.pic.set_pause(paused);
        self.ram.set_pause(paused);
        self.rtc.set_pause(paused);
    }
}
