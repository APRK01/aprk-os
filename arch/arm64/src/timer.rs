// =============================================================================
// APRK OS - ARM Generic Timer
// =============================================================================
// Driver for the ARM64 Generic Timer (ARMv8).
// Uses the Virtual Timer (CNTV) which is standard for guest OSes/VMs.
// =============================================================================

use core::arch::asm;
use core::time::Duration;

pub struct Timer;

impl Timer {
    /// Initialize the timer.
    /// Sets it to fire periodically.
    pub fn init() {
        // disable timer first
        unsafe {
            asm!("msr cntv_ctl_el0, {}", in(reg) 0_u64);
        }
        
        // Schedule the first tick
        Self::set_next_tick(Duration::from_secs(1));
        
        // Enable timer
        // 1 = Enable, 2 = IMASK (0=unmasked, 1=masked)
        // We want Enabled(1) and Unmasked(0) => value 1
        unsafe {
            asm!("msr cntv_ctl_el0, {}", in(reg) 1_u64);
        }
    }

    /// Set the next timer interrupt.
    pub fn set_next_tick(duration: Duration) {
        let freq: u64;
        unsafe {
            asm!("mrs {}, cntfrq_el0", out(reg) freq);
        }
        
        // Calculate ticks properly for any duration
        // ticks = freq * seconds + freq * nanos / 1_000_000_000
        let nanos = duration.as_nanos() as u64;
        let ticks = (freq * nanos) / 1_000_000_000;
        
        // Write to CNTV_TVAL_EL0 (Timer Value Register)
        // This sets the countdown. When it reaches 0, interrupt fires.
        unsafe {
            asm!("msr cntv_tval_el0, {}", in(reg) ticks);
        }
    }
}
