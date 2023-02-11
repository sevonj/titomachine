pub const DEFAULT_MEMSIZE: usize = 1024 * 80;

// State Register Bits
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

pub struct TTKInstance {
    pub memory: Vec<i32>,

    pub cu_pc: i32,     // Program Counter
    pub cu_ir: i32,     // Instruction Register
    pub cu_tr: i32,     // Temporary Regiter
    pub cu_sr: i32,     // State Register
    pub gpr: [i32; 8],  // General Purpose Registers R0..R7
    pub mmu_base: i32,  // -- unimplemented
    pub mmu_limit: i32, // -- unimplemented
    pub mmu_mar: i32,   // Memory address -- unimplemented
    pub mmu_mbr: i32,   // Memory buffer -- unimplemented

    pub running: bool,
    pub halted: bool,
    pub waiting_for_io: bool,
}

impl Default for TTKInstance {
    fn default() -> Self {
        TTKInstance {
            memory: vec![0; DEFAULT_MEMSIZE],
            cu_pc: 0,
            cu_ir: 0,
            cu_tr: 0,
            cu_sr: 0,
            gpr: [0; 8],
            running: false,
            halted: false,
            waiting_for_io: false,
            mmu_base: 0,
            mmu_limit: 0,
            mmu_mar: 0,
            mmu_mbr: 0,
        }
    }
}
