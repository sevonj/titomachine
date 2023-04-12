///
/// devices.rs
///
/// This file handles devices, both memory mapped and port mapped.
/// Every device except CPU is accessible from the Bus struct.
///
///
///
use self::{
    crt::DevCRT, display_classic::DevDisplayClassic, kbd::DevKBD, ram::DevRAM, rtc::DevRTC, pic::DevPIC,
};
mod crt;
mod display_classic;
mod kbd;
mod ram;
mod rtc;
mod pic;

pub(crate) trait Device {
    fn reset(&mut self);
}

pub(crate) trait MMIO {
    fn read(&mut self, addr: usize) -> Result<i32, ()>;
    fn write(&mut self, addr: usize, value: i32) -> Result<(), ()>;
    fn clear(&mut self);
}

pub(crate) trait PMIO {
    fn read_port(&mut self, port: u8) -> Result<i32, ()>;
    fn write_port(&mut self, port: u8, value: i32) -> Result<(), ()>;
}

pub struct Bus {
    pub(crate) ram: DevRAM,
    pub(crate) display: DevDisplayClassic,
    pub(crate) crt: DevCRT,
    pub(crate) kbd: DevKBD,
    pub(crate) rtc: DevRTC,
    pub(crate) pic: DevPIC,
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

    pub(crate) fn reset_devices(&mut self){
        self.pic.reset();
        self.display.reset();
    }
}
