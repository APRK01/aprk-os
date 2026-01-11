// =============================================================================
// APRK OS - CPU Utilities
// =============================================================================
// ARM64 CPU control and information functions.
// =============================================================================

/// Halt the CPU in a low-power state.
/// 
/// This function never returns. It's used when the kernel has nothing to do
/// or when a fatal error occurs.
#[inline(always)]
pub fn halt() -> ! {
    loop {
        // WFE (Wait For Event) puts the CPU in a low-power state until
        // an event or interrupt occurs — but since we're in a loop, we'll
        // just halt forever
        unsafe {
            core::arch::asm!("wfe");
        }
    }
}

/// Enable interrupts.
/// 
/// # Safety
/// Caller must ensure interrupt handlers are properly set up.
#[inline(always)]
pub unsafe fn enable_interrupts() {
    core::arch::asm!("msr daifclr, #2"); // Clear IRQ mask
}

/// Disable interrupts.
#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("msr daifset, #2"); // Set IRQ mask
    }
}

/// Get the current exception level (0-3).
#[inline(always)]
pub fn current_el() -> u8 {
    let el: u64;
    unsafe {
        core::arch::asm!("mrs {}, CurrentEL", out(reg) el);
    }
    ((el >> 2) & 0x3) as u8
}

/// Read the stack pointer.
#[inline(always)]
pub fn read_sp() -> u64 {
    let sp: u64;
    unsafe {
        core::arch::asm!("mov {}, sp", out(reg) sp);
    }

    sp
}

/// Flush the Instruction Cache.
/// Should be called after modifying executable memory.
#[inline(always)]
pub unsafe fn flush_instruction_cache() {
    core::arch::asm!(
        "dsb ish",
        "ic iallu",
        "dsb ish",
        "isb"
    );
}

/// Clean Data Cache by MVA to Point of Unification.
/// Ensures that data written to memory is visible to instruction cache.
pub unsafe fn clean_dcache_range(start: usize, len: usize) {
    let line_size = 64; // Safely assume 64 bytes for Cortex-A72/Virt
    let end = start + len;
    let mut addr = start & !(line_size - 1);
    
    while addr < end {
        core::arch::asm!("dc cvau, {}", in(reg) addr);
        addr += line_size;
    }
    
    core::arch::asm!("dsb ish");
}
