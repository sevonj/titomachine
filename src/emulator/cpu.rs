use super::devices::Bus;

pub mod cpu_debug;
mod instructions;
mod mmu;
mod svc;

////                                    GELOZUMI SPD
//pub const SR_EXCEPTION_MASK: i32 = 0b_00011111_10000000_00000000_00000000;
pub const SR_G: i32 = 1 << 31; // Comp Greater
pub const SR_E: i32 = 1 << 30; // Comp Equal
pub const SR_L: i32 = 1 << 29; // Comp Less

pub const SR_O: i32 = 1 << 28; // Overflow
pub const SR_Z: i32 = 1 << 27; // Zero division
pub const SR_U: i32 = 1 << 26; // Unknown instruction
pub const SR_M: i32 = 1 << 25; // Forbidden mem address

pub const SR_I: i32 = 1 << 24; // Device Interrupt      // unused?
pub const SR_S: i32 = 1 << 23; // SVC
pub const SR_P: i32 = 1 << 22; // Privileged mode       // unused?
pub const SR_D: i32 = 1 << 21; // Disable Interrupts    // unused?

// GPR Names
//pub const R0: usize = 0;
//pub const R1: usize = 1;
//pub const R2: usize = 2;
//pub const R3: usize = 3;
//pub const R4: usize = 4;
//pub const R5: usize = 5;
//pub const R6: usize = 6;
//pub const R7: usize = 7;
pub const SP: usize = 6;
pub const FP: usize = 7;

pub struct CPU {
    // TODO: Do something to these two
    pub input_wait: Option<i32>,
    pub output: Option<(i32, i32)>,
    halt: bool,     //
    burn: bool,     // CPU is disabled permanently.
    cu_pc: i32,     // Program Counter
    cu_ir: i32,     // Instruction Register
    cu_tr: i32,     // Temporary Regiter
    cu_sr: i32,     // State Register
    gpr: [i32; 8],  // General Purpose Registers R0..R7
    mmu_base: u32,  //
    mmu_limit: u32, //
    mmu_mar: u32,   // Mem Address Reg -- unimplemented
    mmu_mbr: i32,   // Mem Buffer Reg -- unimplemented
    ivt: [i32; 16], // Interrupt Vector Table. See comment at exception_check()
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            input_wait: None,
            output: None,
            halt: false,
            burn: false,
            cu_pc: 0,
            cu_ir: 0,
            cu_tr: 0,
            cu_sr: 0,
            gpr: [0; 8],
            mmu_base: 0,
            mmu_limit: u32::MAX,
            mmu_mar: 0,
            mmu_mbr: 0,
            ivt: [0; 16],
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        self.exception_check(bus);
        self.cu_ir = self.memread(bus, self.cu_pc);
        self.cu_pc += 1;
        self.exec_instruction(bus);
    }

    fn exception_check(&mut self, bus: &mut Bus) {
        /*
         * SR    | IVT entry
         *
         * Exceptions:
         *  SR_O | 0: Overflow
         *  SR_Z | 1: Zero div
         *  SR_U | 2: Unknown instruction
         *  SR_M | 3: Forbidden Memory access
         *       | 4: - unused -
         *
         * Device interrupts
         *  SR_I | 5: Memory parity error
         *  SR_I | 6: Timer interrupt
         *  SR_I | 7: Keyboard
         *  SR_I | 8: Mouse
         *  SR_I | 9: Disc drive
         *  SR_I | 10: Printer
         *
         * Supervisor Calls (OS defaults)
         *  SR_S | 11: HALT
         *  SR_S | 12: READ
         *  SR_S | 13: WRITE
         *  SR_S | 14: TIME
         *  SR_S | 15: DATE
         */
        // Interrupts disabled.
        if self.cu_sr & SR_D != 0 {
            return;
        }
        if self.cu_sr & SR_O != 0 {
            self.enter_interrupt_handler(bus, 0);
        }
        if self.cu_sr & SR_Z != 0 {
            self.enter_interrupt_handler(bus, 1);
        }
        if self.cu_sr & SR_U != 0 {
            self.enter_interrupt_handler(bus, 2);
        }
        if self.cu_sr & SR_M != 0 {
            self.enter_interrupt_handler(bus, 3);
        }
        if self.cu_sr & SR_S != 0 {
            // Clear Interrupt flag
            self.cu_sr ^= SR_S;
            match self.cu_tr {
                11 => self.enter_interrupt_handler(bus, 11),
                12 => self.enter_interrupt_handler(bus, 12),
                13 => self.enter_interrupt_handler(bus, 13),
                14 => self.enter_interrupt_handler(bus, 14),
                15 => self.enter_interrupt_handler(bus, 15),
                _ => panic!("svc id wtf {}", self.cu_tr),
            }
        }
    }

    /// Exception handler for traps
    pub(crate) fn exception_trap(&mut self, bus: &mut Bus) {}

    /// Exception handler for device interrupts 
    pub(crate) fn exception_irq(&mut self, bus: &mut Bus) {
        // Interrupts disabled.
        if self.cu_sr & SR_D != 0 {
            return;
        }
        /*
        // idk how this is supposed to work.
        // I assume it goes to tr.
        match self.cu_tr {
            5 => self.enter_interrupt_handler(bus, 5),
            6 => self.enter_interrupt_handler(bus, 6),
            7 => self.enter_interrupt_handler(bus, 7),
            8 => self.enter_interrupt_handler(bus, 8),
            9 => self.enter_interrupt_handler(bus, 9),
            10 => self.enter_interrupt_handler(bus, 10),
            _ => panic!("interrupt id wtf {}", self.cu_tr),
        }
        */

        // Temporary solution: don't check tr at all. We only have a timer interrupt anyway.
        self.enter_interrupt_handler(bus, 6);
    }

    /// Exception handler for service calls
    pub(crate) fn exception_svc(&mut self, bus: &mut Bus) {}

    // Common bookkeeping for interrupt handlers
    fn enter_interrupt_handler(&mut self, bus: &mut Bus, handler_idx: i32) {
        // Push SR, PC, FP
        self.memwrite(bus, self.gpr[SP] + 1, self.cu_sr);
        self.memwrite(bus, self.gpr[SP] + 2, self.cu_pc);
        self.memwrite(bus, self.gpr[SP] + 3, self.gpr[FP]);
        self.gpr[SP] += 3;
        // State flags
        self.cu_sr |= SR_P;
        self.cu_sr |= SR_D;
        // Jump to handler
        self.cu_pc = self.ivt[handler_idx as usize];
    }
}
