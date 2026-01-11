// =============================================================================
// APRK OS - Exception Handling (Rust)
// =============================================================================
// Rust handlers for the exceptions defined in exception.S
// =============================================================================

use crate::println;
use crate::gic::Gic;
use crate::timer::Timer;
use core::time::Duration;

extern "C" {
    fn kernel_syscall_handler(id: u64, arg0: u64, arg1: u64);
}

/// Initialize exceptions.
/// Sets the VBAR_EL1 register to point to our vector table.
pub unsafe fn init() {
    extern "C" {
        static exception_vector_table: u8;
    }
    
    let vector_addr = &exception_vector_table as *const u8 as u64;
    
    // Set VBAR_EL1 (Vector Base Address Register)
    core::arch::asm!("msr vbar_el1, {}", in(reg) vector_addr);
}

/// Handler for Synchronous Exceptions (e.g., Data Abort, SVC).
/// Trap Frame layout matching exception.S SAVE_CONTEXT
#[repr(C)]
pub struct TrapFrame {
    pub x0: u64,   pub x1: u64,   // [sp + 0]
    pub x2: u64,   pub x3: u64,   // [sp + 16]
    pub x4: u64,   pub x5: u64,   // [sp + 32]
    pub x6: u64,   pub x7: u64,   // [sp + 48]
    pub x8: u64,   pub x9: u64,   // [sp + 64]
    pub x10: u64,  pub x11: u64,  // [sp + 80]
    pub x12: u64,  pub x13: u64,  // [sp + 96]
    pub x14: u64,  pub x15: u64,  // [sp + 112]
    pub x16: u64,  pub x17: u64,  // [sp + 128]
    pub x18: u64,  pub x19: u64,  // [sp + 144]
    pub x20: u64,  pub x21: u64,  // [sp + 160]
    pub x22: u64,  pub x23: u64,  // [sp + 176]
    pub x24: u64,  pub x25: u64,  // [sp + 192]
    pub x26: u64,  pub x27: u64,  // [sp + 208]
    pub x28: u64,  pub x29: u64,  // [sp + 224]
    pub x30: u64,                 // [sp + 240] (LR)
}

/// Handler for Synchronous Exceptions (SVC, Data Abort, etc.).
/// 
/// `trap_frame` points to the saved register context on the stack.
#[no_mangle]
pub extern "C" fn handle_sync_exception(trap_frame: *mut TrapFrame) {
    let esr: u64;
    
    unsafe {
        core::arch::asm!("mrs {}, esr_el1", out(reg) esr);
    }
    
    let ec = (esr >> 26) & 0x3F;

    // EC = 0x15 is SVC (System Call) from AArch64
    if ec == 0x15 {
        // Read syscall arguments from the saved trap frame
        let tf = unsafe { &*trap_frame };
        let id = tf.x8;    // Syscall number in x8
        let arg0 = tf.x0;  // First argument in x0
        let arg1 = tf.x1;  // Second argument in x1
        
        unsafe {
            kernel_syscall_handler(id, arg0, arg1);

            // Advance ELR_EL1 by 4 bytes to skip the SVC instruction
            let mut elr: u64;
            core::arch::asm!("mrs {0}, elr_el1", out(reg) elr);
            elr += 4;
            core::arch::asm!("msr elr_el1, {0}", in(reg) elr);
        }
        return; // Return to user
    }
    
    let elr: u64;
    unsafe {
        core::arch::asm!("mrs {}, elr_el1", out(reg) elr);
    }
    
    println!("\n!!! SYNCHRONOUS EXCEPTION !!!");
    println!("ESR_EL1: {:#018x}", esr);
    println!("ELR_EL1: {:#018x}", elr);
    println!("System halted.");
    
    loop { core::hint::spin_loop(); }
}

/// Handler for IRQ Exceptions (Hardware Interrupts).
#[no_mangle]
pub extern "C" fn handle_irq_exception() {
    // 1. Acknowledge interrupt from GIC
    let iar = Gic::acknowledge();
    let irq_id = iar & 0x3FF; // Lower 10 bits are the ID

    // 2. Handle the interrupt
    match irq_id {
        27 | 30 => {
            // Timer Interrupt
            // CRITICAL: Rearm timer and EOI BEFORE kernel_tick because 
            // kernel_tick may context switch and never return!
            Timer::set_next_tick(Duration::from_millis(500)); // 500ms timer tick
            Gic::end_interrupt(iar);
            
            extern "Rust" { fn kernel_tick(); }
            unsafe { kernel_tick(); }
            return; // EOI already done above
        }
        33 => {
            // UART Interrupt
            crate::uart::handle_irq();
        }
        1023 => {
            // Spurious - ignore
            return; // Don't EOI spurious
        }
        _ => {
            println!("[IRQ] Unknown interrupt ID: {}", irq_id);
        }
    }

    // 3. Signal End Of Interrupt to GIC
    Gic::end_interrupt(iar);
}
