pub mod cpu_debug;
mod instructions;
mod mmu;
mod svc;

pub const DEFAULT_MEMSIZE: usize = 1024 * 80;

// State Register
pub const SR_DEFAULT: i32 = 0;
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
pub const R0: usize = 0;
pub const R1: usize = 1;
pub const R2: usize = 2;
pub const R3: usize = 3;
pub const R4: usize = 4;
pub const R5: usize = 5;
pub const R6: usize = 6;
pub const R7: usize = 7;
pub const SP: usize = 6;
pub const FP: usize = 7;

pub struct CPU {
    // These two will be removed or something
    pub waiting_for_io: bool,
    //pub running: bool,
    halt: bool,       //
    cu_pc: i32,       // Program Counter
    cu_ir: i32,       // Instruction Register
    cu_tr: i32,       // Temporary Regiter
    cu_sr: i32,       // State Register
    gpr: [i32; 8],    // General Purpose Registers R0..R7
    mmu_base: u32,    //
    mmu_limit: u32,   //
    mmu_mar: u32,     // Mem Address Reg -- unimplemented
    mmu_mbr: i32,     // Mem Buffer Reg -- unimplemented
    memory: Vec<i32>, // Memory being inside the CPU is kinda dumb, but hey.
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            waiting_for_io: false,
            halt: false,
            cu_pc: 0,
            cu_ir: 0,
            cu_tr: 0,
            cu_sr: 0,
            gpr: [0; 8],
            mmu_base: 0,
            mmu_limit: u32::MAX,
            mmu_mar: 0,
            mmu_mbr: 0,
            memory: vec![0; DEFAULT_MEMSIZE],
        }
    }

    pub fn tick(&mut self) {
        self.cu_ir = self.memread(self.cu_pc);
        self.exec_instruction();
        self.sr_handler()
    }

    // Check for anomalies in state registers
    fn sr_handler(&mut self) {
        if self.cu_sr & SR_S != 0 {
            self.svc_handler();
        }
        if self.cu_sr & SR_M != 0 {
            println!("Program Error: Forbidden memory address!");
            self.debug_set_halt(true);
        }
        if self.cu_sr & SR_U != 0 {
            println!("Program Error: Unknown Instruction!");
            self.debug_set_halt(true);
        }
        if self.cu_sr & SR_Z != 0 {
            println!("Program Error: Zero division!");
            self.debug_set_halt(true);
        }
        if self.cu_sr & SR_O != 0 {
            println!("Program Error: Overflow!");
            self.debug_set_halt(true);
        }
    }

    pub fn input_handler(&mut self, input: i32) {
        if self.waiting_for_io == false {
            panic!("input_handler(): waiting_for_io is false. Why did you call me?")
        }
        let rj = (self.cu_ir >> 21) & 0x7;
        self.gpr[rj as usize] = input;
        self.waiting_for_io = false;
        self.cu_pc += 1;
    }
}
