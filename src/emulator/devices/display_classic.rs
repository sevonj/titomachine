use std::{sync::mpsc::Sender, time::Duration};

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
    tx: Option<Sender<Vec<Rgba<u8>>>>,
    framebuffer: Vec<Rgba<u8>>,
    frame_timer: Duration,
    frame_rate: u32,
}
impl Default for DevDisplayClassic {
    fn default() -> Self {
        Self {
            tx: None,
            framebuffer: vec![image::Rgba([0, 0, 0, 255,]); 120 * 160],
            frame_timer: Duration::ZERO,
            frame_rate: 60,
        }
    }
}

impl Device for DevDisplayClassic {
    fn reset(&mut self) {
        self.framebuffer = vec![image::Rgba([0, 0, 0, 255,]); 120 * 160];
    }
}

impl DevDisplayClassic {
    pub fn connect(&mut self, tx: Sender<Vec<Rgba<u8>>>) {
        self.tx = Some(tx);
    }
    pub fn update(&mut self, t_delta: Duration) {
        self.frame_timer += t_delta;
        let frame_time = Duration::from_secs(1) / self.frame_rate;
        if self.frame_timer >= frame_time {
            self.frame_timer -= frame_time;
            self.send();
        }
    }
    pub(crate) fn send(&mut self) {
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
