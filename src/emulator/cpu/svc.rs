/*
///
/// cpu/svc.rs
/// 
/// These are the old faux SVC functions.
/// Possibly re-add them as an option?
/// 

use chrono::{Datelike, Local, Timelike};
use num_traits::FromPrimitive;

use super::{CPU, SP, SR_S};

#[derive(FromPrimitive, ToPrimitive)]
enum SVC {
    HALT = 11,
    READ = 12,
    WRITE = 13,
    TIME = 14,
    DATE = 15,
}

impl CPU {
    pub fn svc_handler(&mut self) {
        if self.cu_sr & SR_S == 0 {
            return;
        }
        self.cu_sr &= !SR_S; // Clear syscall flag

        match FromPrimitive::from_i32(self.cu_tr) {
            Some(SVC::HALT) => {
                println!("SVC: System halted.");
                self.halt = true;
            }
            Some(SVC::READ) => {
                println!("SVC: ERR: READ not implemented!");
                self.halt = true;
            }
            Some(SVC::WRITE) => {
                println!("SVC: ERR: WRITE not implemented!");
                self.halt = true;
            }
            Some(SVC::TIME) => {
                let s_addr = self.memread(self.gpr[SP]);
                let m_addr = self.memread(self.gpr[SP] - 1);
                let h_addr = self.memread(self.gpr[SP] - 2);
                self.gpr[SP] -= 3;
                self.memwrite(s_addr, Local::now().second() as i32);
                self.memwrite(m_addr, Local::now().minute() as i32);
                self.memwrite(h_addr, Local::now().hour() as i32);
            }
            Some(SVC::DATE) => {
                let d_addr = self.memread(self.gpr[SP]);
                let m_addr = self.memread(self.gpr[SP] - 1);
                let y_addr = self.memread(self.gpr[SP] - 2);
                self.gpr[SP] -= 3;
                self.memwrite(d_addr, Local::now().day() as i32);
                self.memwrite(m_addr, Local::now().month() as i32);
                self.memwrite(y_addr, Local::now().year() as i32);
            }
            _ => {
                println!("SVC: ERR: Unknown request!");
                self.halt = true;
            }
        }
    }
}*/
