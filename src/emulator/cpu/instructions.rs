use super::CPU;
#[allow(unused_imports)] // Some of these flags are unused.
use super::{GPR, SR_D, SR_E, SR_G, SR_I, SR_L, SR_M, SR_O, SR_P, SR_S, SR_U, SR_Z};
use crate::emulator::Bus;

const NOP: u16 = 0x00;
const STORE: u16 = 0x01;
const LOAD: u16 = 0x02;
const IN: u16 = 0x03;
const OUT: u16 = 0x04;
const ADD: u16 = 0x11;
const SUB: u16 = 0x12;
const MUL: u16 = 0x13;
const DIV: u16 = 0x14;
const MOD: u16 = 0x15;
const AND: u16 = 0x16;
const OR: u16 = 0x17;
const XOR: u16 = 0x18;
const SHL: u16 = 0x19;
const SHR: u16 = 0x1A;
const NOT: u16 = 0x1B;
const SHRA: u16 = 0x1C;
const COMP: u16 = 0x1F;
const JUMP: u16 = 0x20;
const JNEG: u16 = 0x21;
const JZER: u16 = 0x22;
const JPOS: u16 = 0x23;
const JNNEG: u16 = 0x24;
const JNZER: u16 = 0x25;
const JNPOS: u16 = 0x26;
const JLES: u16 = 0x27;
const JEQU: u16 = 0x28;
const JGRE: u16 = 0x29;
const JNLES: u16 = 0x2A;
const JNEQU: u16 = 0x2B;
const JNGRE: u16 = 0x2C;
const CALL: u16 = 0x31;
const EXIT: u16 = 0x32;
const PUSH: u16 = 0x33;
const POP: u16 = 0x34;
const PUSHR: u16 = 0x35;
const POPR: u16 = 0x36;
const IEXIT: u16 = 0x39;
const SVC: u16 = 0x70;
const HLT: u16 = 0x71;
const HCF: u16 = 0x72;

impl CPU {
    pub(crate) fn exec_instruction(&mut self, bus: &mut Bus) {
        let opcode = (self.cu_ir >> 24) as u16;
        let rj = (self.cu_ir >> 21) & 0x7;
        let mode = (self.cu_ir >> 19) & 0x3;
        let ri = (self.cu_ir >> 16) & 0x7;
        // these casts catch the sign
        let addr = (self.cu_ir & 0xffff) as i16 as i32;

        match self.fetch_second_operand(bus, mode, ri, addr) {
            Ok(val) => self.cu_tr = val,
            Err(_) => return
        }

        match opcode {
            NOP => return,
            STORE => {
                let _ = self.memwrite(bus, self.cu_tr, self.gpr[rj as usize]);
            }
            LOAD => self.gpr[rj as usize] = self.cu_tr,
            IN => match bus.read_port(self.cu_tr) {
                Ok(val) => self.gpr[rj as usize] = val,
                Err(_) => return
            }
            OUT => {
                let _ = bus.write_port(self.cu_tr, self.gpr[rj as usize]);
            }
            ADD => match self.gpr[rj as usize].checked_add(self.cu_tr) {
                Some(i) => self.gpr[rj as usize] = i,
                None => self.exception_trap_o(bus),
            },
            SUB => match self.gpr[rj as usize].checked_sub(self.cu_tr) {
                Some(i) => self.gpr[rj as usize] = i,
                None => self.exception_trap_o(bus),
            },
            MUL => match self.gpr[rj as usize].checked_mul(self.cu_tr) {
                Some(i) => self.gpr[rj as usize] = i,
                None => self.exception_trap_o(bus),
            },
            DIV => {
                if self.cu_tr == 0 {
                    return self.exception_trap_z(bus);
                }
                match self.gpr[rj as usize].checked_div(self.cu_tr) {
                    Some(i) => self.gpr[rj as usize] = i,
                    None => self.exception_trap_o(bus),
                }
            }
            MOD => self.gpr[rj as usize] %= self.cu_tr,
            AND => self.gpr[rj as usize] &= self.cu_tr,
            OR => self.gpr[rj as usize] |= self.cu_tr,
            XOR => self.gpr[rj as usize] ^= self.cu_tr,
            SHL => self.gpr[rj as usize] <<= self.cu_tr,
            SHR => self.gpr[rj as usize] = (self.gpr[rj as usize] as u32 >> self.cu_tr) as i32,
            NOT => self.gpr[rj as usize] = !self.gpr[rj as usize],
            SHRA => {
                self.gpr[rj as usize] = self.gpr[rj as usize]
                    .checked_shr(self.cu_tr as u32)
                    .unwrap_or(match self.gpr[rj as usize] >= 0 {
                        true => 0,
                        false => -1,
                    })
            }
            COMP => {
                if self.gpr[rj as usize] > self.cu_tr {
                    // Greater
                    self.cu_sr |= SR_G;
                    self.cu_sr &= !SR_E;
                    self.cu_sr &= !SR_L;
                } else if self.gpr[rj as usize] == self.cu_tr {
                    // Equal
                    self.cu_sr &= !SR_G;
                    self.cu_sr |= SR_E;
                    self.cu_sr &= !SR_L;
                } else {
                    // Less
                    self.cu_sr &= !SR_G;
                    self.cu_sr &= !SR_E;
                    self.cu_sr |= SR_L;
                }
            }
            // Branching instructions
            JUMP => self.cu_pc = self.cu_tr,
            // Jumps that use GPR
            JNEG => {
                if self.gpr[rj as usize] < 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JZER => {
                if self.gpr[rj as usize] == 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JPOS => {
                if self.gpr[rj as usize] > 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNNEG => {
                if self.gpr[rj as usize] >= 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNZER => {
                if self.gpr[rj as usize] != 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNPOS => {
                if self.gpr[rj as usize] <= 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            // Jumps that use SR
            JLES => {
                if self.cu_sr & SR_L == SR_L {
                    self.cu_pc = self.cu_tr;
                }
            }
            JEQU => {
                if self.cu_sr & SR_E == SR_E {
                    self.cu_pc = self.cu_tr;
                }
            }
            JGRE => {
                if self.cu_sr & SR_G == SR_G {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNLES => {
                if self.cu_sr & SR_L == 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNEQU => {
                if self.cu_sr & SR_E == 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            JNGRE => {
                if self.cu_sr & SR_G == 0 {
                    self.cu_pc = self.cu_tr;
                }
            }
            // Subroutine instructions
            CALL => {
                if let Err(_) = self.memwrite(bus, self.gpr[GPR::SP as usize] + 1, self.cu_pc)
                {
                    return;
                };
                if let Err(_) = self.memwrite(bus, self.gpr[GPR::SP as usize] + 2, self.gpr[GPR::FP as usize]) {
                    return;
                };
                self.gpr[GPR::SP as usize] += 2;
                self.cu_pc = self.cu_tr;
                self.gpr[GPR::FP as usize] = self.gpr[GPR::SP as usize];
            }
            EXIT => {
                self.gpr[GPR::SP as usize] = self.gpr[GPR::FP as usize] - 2 - self.cu_tr;
                match self.memread(bus, self.gpr[GPR::FP as usize] - 1) {
                    Ok(val) => self.cu_pc = val,
                    Err(_) => return
                }
                match self.memread(bus, self.gpr[GPR::FP as usize]) {
                    Ok(val) => self.gpr[GPR::FP as usize] = val,
                    Err(_) => return
                }
            }
            // Stack instructions
            PUSH => {
                self.gpr[GPR::SP as usize] += 1;
                let _ = self.memwrite(bus, self.gpr[GPR::SP as usize], self.cu_tr);
            }
            POP => {
                match self.memread(bus, self.gpr[GPR::SP as usize]) {
                    Ok(val) => self.gpr[ri as usize] = val,
                    Err(_) => return
                }
                self.gpr[GPR::SP as usize] -= 1;
            }
            PUSHR => {
                for i in 0..7 {
                    self.gpr[GPR::SP as usize] += 1;
                    if let Err(_) = self.memwrite(bus, self.gpr[GPR::SP as usize], self.gpr[i]) {
                        return;
                    }
                }
            }
            POPR => {
                let old_sp = self.gpr[GPR::SP as usize];
                for i in (0..7).rev() {
                    let addr;
                    match old_sp.checked_sub(6) {
                        Some(n) => match n.checked_add(i) {
                            Some(n) => addr = n,
                            None => return,
                        },
                        None => return,
                    }
                    match self.memread(bus, addr) {
                        Ok(val) => self.gpr[i as usize] = val,
                        Err(_) => return
                    }
                    self.gpr[GPR::SP as usize] -= 1;
                }
            }
            IEXIT => {
                // Pop FP, PC, SR
                match self.memread(bus, self.gpr[GPR::SP as usize]) {
                    Ok(val) => self.gpr[GPR::FP as usize] = val,
                    Err(_) => return
                }
                match self.memread(bus, self.gpr[GPR::SP as usize] - 1) {
                    Ok(val) => self.cu_pc = val,
                    Err(_) => return
                }
                match self.memread(bus, self.gpr[GPR::SP as usize] - 2) {
                    Ok(val) => self.cu_sr = val,
                    Err(_) => return
                }
                self.gpr[GPR::SP as usize] -= 3;
                // Pop params
                self.gpr[GPR::SP as usize] -= self.cu_tr;
            }
            // Syscalls
            SVC => self.exception_svc(bus),
            HLT => self.halt = true,
            HCF => {
                self.halt = true;
                self.burn = true;
                println!("Execution has ended.");
            }
            _ => self.exception_trap_u(bus),
        }
    }

    fn fetch_second_operand(
        &mut self,
        bus: &mut Bus,
        mode: i32,
        ri: i32,
        addr: i32,
    ) -> Result<i32, ()> {

        // Value of second register.
        let ri_val = match ri {
            // R0, zero regardless
            0 => 0,
            // Not R0, use value of the register.
            _ => self.gpr[ri as usize],
        };

        // Immediate = address + ri_val
        let immediate = match addr.checked_add(ri_val) {
            Some(i) => i,
            None => {
                self.exception_trap_o(bus);
                return Err(());
            }
        };

        // Return value through relevant number of memory fetches.
        match mode {
            // No fetch
            0 => Ok(immediate),
            // 1 fetch
            1 => Ok(self.memread(bus, immediate)?),
            // 2 fetches
            2 => {
                let ptr = self.memread(bus, immediate)?;
                Ok(self.memread(bus, ptr)?)
            }
            _ => {
                self.exception_trap_u(bus);
                Err(())
            }
        }
    }
}
