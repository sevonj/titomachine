use super::CPU;
use crate::emulator::cpu::SR_M;

impl CPU {
    pub fn virtual2real(&mut self, addr: i32) -> Result<usize, ()> {
        // Check Base / Limit
        if addr as u32 >= self.mmu_limit {
            self.cu_sr |= SR_M;
            return Err(());
        }
        let real_addr = addr as usize + self.mmu_base as usize;
        // Check that real addr fits in memory
        if self.memory.len() <= real_addr {
            self.cu_sr |= SR_M;
            return Err(());
        }
        return Ok(real_addr as usize);
    }

    pub fn memread(&mut self, addr: i32) -> i32 {
        match self.virtual2real(addr) {
            Err(_) => return 0,
            Ok(real_addr) => self.memory[real_addr],
        }
    }

    pub fn memwrite(&mut self, addr: i32, value: i32) {
        match self.virtual2real(addr) {
            Err(_) => return,
            Ok(real_addr) => self.memory[real_addr] = value,
        }
    }
}
