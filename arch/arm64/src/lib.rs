// =============================================================================
// APRK OS - ARM64 Architecture Module
// =============================================================================
// This module contains all ARM64-specific code:
// - UART driver for console output
// - Boot initialization
// - CPU utilities
// - Exception handling
// - Interrupt Controller
// - Timer
// - MMU
//
// SPDX-License-Identifier: GPL-2.0
// =============================================================================

#![no_std]

pub mod uart;
pub mod cpu;
pub mod exception;
pub mod gic;
pub mod timer;
pub mod mmu;
pub mod context;

/// Initialize the ARM64 hardware for kernel operation.
/// 
/// This function is called early in the boot process to set up
/// essential hardware before the kernel can run properly.
/// 
/// # Safety
/// This function must only be called once during boot.
pub fn init() {
    // 1. Initialize UART (for debug output)
    uart::init();
    
    // 2. Initialize MMU (enable virtual memory & caches)
    // SAFETY: We trust our page table setup is correct
    unsafe { mmu::init(); }
    
    // 3. Initialize Exception Vectors
    unsafe { exception::init(); }
    
    // 4. Initialize GIC (Interrupt Controller)
    unsafe { gic::Gic::init(); }
    
    // 5. Initialize Timer
    timer::Timer::init();
    
    // 6. Enable Interrupts (CPU level)
    unsafe { cpu::enable_interrupts(); }
}
