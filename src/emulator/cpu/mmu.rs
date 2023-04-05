///
/// cpu/mmu.rs
///
/// All Memory access goes through here.
///
use super::CPU;
use crate::emulator::{cpu::SR_M, Bus};

impl CPU {
    fn virtual2real(&mut self, addr: i32) -> Result<u32, ()> {
        if addr as u32 >= self.mmu_limit {
            self.cu_sr |= SR_M;
            return Err(());
        }
        return Ok(addr as u32 + self.mmu_base);
    }

    pub(crate) fn memread(&mut self, bus: &mut Bus, addr: i32) -> i32 {
        if let Ok(real_addr) = self.virtual2real(addr) {
            if let Ok(val) = bus.read(real_addr) {
                return val;
            }
        }
        self.cu_sr |= SR_M;
        0
    }

    pub(crate) fn memwrite(&mut self, bus: &mut Bus, addr: i32, value: i32) {
        if let Ok(real_addr) = self.virtual2real(addr) {
            if let Err(_) = bus.write(real_addr, value) {
                self.cu_sr |= SR_M;
            };
        }
    }
}
