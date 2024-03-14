///
/// cpu/mmu.rs
///
/// All Memory access goes through here.
///
use super::CPU;
use crate::emulator::Bus;

impl CPU {
    /// Address conversion & protection check
    fn virtual2real(&mut self, addr: i32) -> Result<u32, ()> {
        if addr as u32 >= self.mmu_limit {
            return Err(());
        }
        Ok(addr as u32 + self.mmu_base)
    }

    pub(crate) fn memread(&mut self, bus: &mut Bus, addr: i32) -> Result<i32, ()> {
        let real_addr;
        match self.virtual2real(addr) {
            Ok(val) => real_addr = val,
            Err(_) => {
                self.exception_trap_m(bus);
                return Err(());
            }
        }
        match bus.read(real_addr) {
            Ok(val) => Ok(val),
            Err(_) => {
                self.exception_trap_m(bus);
                return Err(());
            }
        }
    }

    pub(crate) fn memwrite(&mut self, bus: &mut Bus, addr: i32, value: i32) -> Result<(), ()> {
        let real_addr;
        match self.virtual2real(addr) {
            Ok(val) => real_addr = val,
            Err(_) => {
                self.exception_trap_m(bus);
                return Err(());
            }
        }
        match bus.write(real_addr, value) {
            Ok(_) => Ok(()),
            Err(_) => {
                self.exception_trap_m(bus);
                return Err(());
            }
        }
    }
}
