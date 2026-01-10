// =============================================================================
// APRK OS - Kernel Entry Point
// =============================================================================
// This is the main Rust entry point for the APRK OS kernel.
// Called from boot.S after basic hardware initialization.
//
// SPDX-License-Identifier: GPL-2.0
// Copyright (c) 2025 APRK
// =============================================================================

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use aprk_arch_arm64::{self as arch, cpu, println};
use alloc::vec::Vec;
use core::panic::PanicInfo;

mod mm;
mod sched;
mod shell;

// Task 1 Function (Replaced by Shell)
// extern "C" fn task_one() { ... }


// User Process (EL0)
extern "C" fn user_land() {
    loop {
        // SYSCALL (SVC)
        unsafe {
            core::arch::asm!(
                "mov x8, #42",
                "svc #0",
                options(nostack)
            );
            
            for _ in 0..10_000_000 { core::arch::asm!("nop") }
        }
    }
}

// Task 2 Function (Kernel Wrapper for User Mode)
extern "C" fn task_two() {
    unsafe { arch::cpu::enable_interrupts(); }
    
    println!("Task 2: Dropping to EL0 (User Mode)...");
    
    // We need a stack for EL0. We'll just use the current stack base?
    // Current stack is growing down.
    // Let's allocate a small buffer on stack? No, that's dangerous.
    // Let's grab a new page.
    // But we don't have access to heap here easily without `alloc`? We do.
    
    let user_stack = Vec::<u8>::with_capacity(4096);
    let user_stack_top = user_stack.as_ptr() as u64 + 4096;
    
    // Forget the vec so it doesn't free
    core::mem::forget(user_stack);
    
    unsafe {
        arch::context::enter_user_mode(user_land as *const () as u64, user_stack_top);
    }
}
// Version Information
// =============================================================================

/// APRK OS version
const VERSION: &str = "0.0.1";

/// APRK OS codename
const CODENAME: &str = "Genesis";

// =============================================================================
// Kernel Entry Point
// =============================================================================

/// Kernel main entry point.
/// 
/// This function is called from assembly boot code after:
/// - CPU 0 is selected (other cores are halted)
/// - Stack is initialized
/// - BSS section is zeroed
/// 
/// # Safety
/// This function must be called only once, by the boot assembly code.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    // Initialize architecture-specific hardware
    // This now includes MMU, Exceptions, GIC, and Timer!
    arch::init();
    
    // Initialize Memory Management (PMM + Heap)
    mm::init();

    // Print the APRK OS banner
    print_banner();

    // Print system information
    print_system_info();

    println!();
    println!("[kernel] Core subsystems initialized:");
    println!("         - MMU (Identity Map)");
    println!("         - Exceptions (Vector Table)");
    println!("         - GICv2 (Interrupt Controller)");
    println!("         - ARM Generic Timer");
    println!("         - PMM & Heap Allocator");
    
    // Initialize Scheduler
    sched::init();
    sched::spawn(shell::run);
    // sched::spawn(task_two);
    
    // Test Heap
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    println!("[kernel] Heap Test: Vec = {:?}", v);
    
    println!();
    println!("[kernel] Waiting for timer interrupts... (Press Ctrl+A, X to exit)");

    // Enter main loop - we just wait for interrupts now
    loop {
        // Wait For Event - puts CPU to sleep until interrupt fires
        unsafe { core::arch::asm!("wfe"); }
    }
}

// Timer Callback
#[no_mangle]
pub extern "Rust" fn kernel_tick() {
    sched::schedule();
}

// =============================================================================
// Boot Output
// =============================================================================

/// Print the APRK OS boot banner.
fn print_banner() {
    println!();
    println!(r"    _    ____  ____  _  __   ___  ____  ");
    println!(r"   / \  |  _ \|  _ \| |/ /  / _ \/ ___| ");
    println!(r"  / _ \ | |_) | |_) | ' /  | | | \___ \ ");
    println!(r" / ___ \|  __/|  _ <| . \  | |_| |___) |");
    println!(r"/_/   \_\_|   |_| \_\_|\_\  \___/|____/ ");
    println!();
    println!("APRK OS v{} \"{}\"", VERSION, CODENAME);
    println!("A modern operating system kernel for ARM64");
    println!();
    println!("============================================================");
}

/// Print system information.
fn print_system_info() {
    let el = cpu::current_el();
    let sp = cpu::read_sp();

    println!("[boot] Kernel loaded successfully");
    println!("[boot] Current Exception Level: EL{}", el);
    println!("[boot] Stack Pointer: {:#018x}", sp);
    println!("[boot] UART initialized");
}

// =============================================================================
// Panic Handler
// =============================================================================

/// Panic handler for kernel panics.
/// 
/// This is called when the kernel encounters an unrecoverable error.
/// We print diagnostic information and halt the CPU.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!();
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!("!!                     KERNEL PANIC                        !!");
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!();
    
    if let Some(location) = info.location() {
        println!("Location: {}:{}:{}", 
            location.file(), 
            location.line(), 
            location.column()
        );
    }
    
    if let Some(message) = info.message().as_str() {
        println!("Message: {}", message);
    } else {
        println!("Message: {}", info.message());
    }
    
    println!();
    println!("System halted.");
    
    cpu::halt();
}

// =============================================================================
// Tests (for future use)
// =============================================================================

#[cfg(test)]
mod tests {
    // Unit tests will go here
}
