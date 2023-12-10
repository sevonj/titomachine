//!
//! Memory mapped framebuffer
//!
use super::{Device, MMIO};
use image::Rgba;
use std::sync::mpsc::Sender;

/// Color screen with memory mapped framebuffer
/// It displays the image identically to titokone.
///
/// 160x120
pub(crate) struct DevDisplayClassic {
    /// mpsc sender for framebuf
    tx: Option<Sender<Vec<Rgba<u8>>>>,
    framebuffer: Vec<Rgba<u8>>,
    /// Interrupt signal
    pub(crate) interrupt: bool,
}
impl Default for DevDisplayClassic {
    fn default() -> Self {
        Self {
            tx: None,
            framebuffer: vec![image::Rgba([0, 0, 0, 255,]); 120 * 160],
            interrupt: false,
        }
    }
}
impl Device for DevDisplayClassic {
    fn reset(&mut self) {
        self.framebuffer = vec![image::Rgba([0, 0, 0, 255,]); 120 * 160];
        self.interrupt = false
    }
    fn on(&mut self) {}
    fn off(&mut self) {
        self.framebuffer = vec![image::Rgba([0, 0, 0, 255,]); 120 * 160];
        self.interrupt = false
    }
}

impl DevDisplayClassic {
    /// Give the device an mpsc sender to send framebuffer to.
    pub fn connect(&mut self, tx: Sender<Vec<Rgba<u8>>>) {
        self.tx = Some(tx);
    }
    /// Send framebuffer
    pub(crate) fn send(&mut self) {
        self.interrupt = true;
        if let Some(tx) = &self.tx {
            tx.send(self.framebuffer.clone());
        }
    }
}

impl MMIO for DevDisplayClassic {
    fn read(&mut self, addr: usize) -> Result<i32, ()> {
        if addr >= self.framebuffer.len() {
            return Err(());
        }
        let color = self.framebuffer[addr];
        Ok((color[0] << 4) as i32 + (color[1]) as i32 + (color[2] >> 4) as i32)
    }
    fn write(&mut self, addr: usize, value: i32) -> Result<(), ()> {
        if addr >= self.framebuffer.len() {
            return Err(());
        }
        let color = Rgba([(value >> 4) as u8, (value) as u8, (value << 4) as u8, 255]);
        self.framebuffer[addr] = color;
        Ok(())
    }
}
