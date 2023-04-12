///
/// devices/display_classic.rs
///
/// Color screen with memory mapped framebuffer.
/// It displays the image identically to titokone.
/// 160x120, 3 least significant bytes map to RGB
/// Reading from the framebuffer is allowed.
/// When writing, information outside RGB bytes is lost.
///
use super::{Device, MMIO};
use image::Rgba;

pub(crate) struct DevDisplayClassic {
    framebuffer: Vec<Rgba<u8>>,
}

impl Default for DevDisplayClassic {
    fn default() -> Self {
        DevDisplayClassic {
            framebuffer: vec![image::Rgba([0, 0, 0, 255,]); 120 * 160],
        }
    }
}
impl Device for DevDisplayClassic {
    fn reset(&mut self) {
        self.framebuffer = vec![image::Rgba([0, 0, 0, 255,]); 120 * 160];
    }
}

impl DevDisplayClassic {
    pub(crate) fn debug_get_framebuf(&mut self) -> Vec<Rgba<u8>> {
        self.framebuffer.clone()
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
    fn clear(&mut self) {
        self.framebuffer = vec![image::Rgba([0, 0, 0, 255,]); 120 * 160];
    }
}
