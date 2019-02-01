#[derive(Clone, Copy)]
pub enum ProcessorMode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Supervisor = 0b10011,
    Monitor = 0b10110,
    Abort = 0b10111,
    Hypervisor = 0b11010,
    Undefined = 0b11011,
    System = 0b11111,
}

pub fn get_cpsr() -> u32 {
    unsafe {
        let cpsr;
        asm!("mrs $0, cpsr" : "=r"(cpsr));
        cpsr
    }
}

pub fn get_spsr() -> u32 {
    unsafe {
        let spsr;
        asm!("mrs $0, spsr" : "=r"(spsr));
        spsr
    }
}

// Note: This function must be called only with compile time constant values
// or it will generate a compilation error.
#[inline(always)]
pub unsafe fn set_mode(mode: ProcessorMode) {
    asm!("cps $0" :: "i"(mode as u32) :: "volatile");
}

coproc_reg! {
    SCTLR : p15, c1, 0, c0, 0;
    CPACR : p15, c1, 0, c0, 2;
}

bitflags! {
    pub struct Features : u32 {
        const MMU = 1 << 0;
        const ALIGNMENT_CHECK = 1 << 1;
        const CACHE = 1 << 2;
        const SWP_INSTRUCTION = 1 << 10;
        const BRANCH_PREDICTION = 1 << 11;
        const INSTRUCTION_CACHE = 1 << 12;
        const TEX_REMAP = 1 << 28;
        const ACCESS_FLAG = 1 << 29;
    }
}

pub unsafe fn enable_features(features: Features) {
    SCTLR::set_bits(features.bits());
}

pub unsafe fn disable_features(features: Features) {
    SCTLR::reset_bits(features.bits());
}

pub fn enable_fpu() {
    unsafe {
        CPACR::write(0b1111 << 20);
        asm!("vmsr FPEXC, $0" :: "r"(1 << 30) :: "volatile");
    }
}

pub fn disable_fpu() {
    unsafe {
        asm!("vmsr FPEXC, $0" :: "r"(0) :: "volatile");
        CPACR::write(0);
    }
}
