///
/// cpu/mmu.rs
///
/// Memory Management Unit
///
use super::CPU;
use crate::emulator::{cpu::SR_M, Bus};

impl CPU {
    /// Convert virtual address to real address:
    /// this does the base/limit address management.
    pub fn virtual2real(&mut self, addr: i32) -> Result<i32, ()> {
        if addr as u32 >= self.mmu_limit {
            self.cu_sr |= SR_M;
            return Err(());
        }
        let real_addr = addr as u32 + self.mmu_base;
        return Ok(real_addr as i32);
    }

    pub fn memread(&mut self, bus: &mut Bus, addr: i32) -> i32 {
        match self.virtual2real(addr) {
            Err(_) => return 0,
            Ok(real_addr) => {
                if let Ok(val) = bus.read(real_addr) {
                    val
                } else {
                    self.cu_sr |= SR_M;
                    0
                }
            }
        }
    }

    pub fn memwrite(&mut self, bus: &mut Bus, addr: i32, value: i32) {
        match self.virtual2real(addr) {
            Err(_) => return,
            Ok(real_addr) => {
                if let Err(_) = bus.write(real_addr, value) {
                    self.cu_sr |= SR_M;
                };
            }
        }
    }
}
