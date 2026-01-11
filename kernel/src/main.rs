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

use aprk_arch_arm64::{self as arch, cpu, print, println};
use alloc::vec::Vec;
use core::panic::PanicInfo;

mod mm;
mod sched;
mod shell;
pub mod fs;
mod loader;

// Task 1 Function (Replaced by Shell)
// extern "C" fn task_one() { ... }


// =============================================================================
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
    sched::spawn_named(shell::run, "shell", sched::Priority::High);
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

// Syscall Handler
// Syscall Numbers:
//   0: print(ptr, len) - Print string to console
//   1: exit()          - Terminate current process
//   2: getpid()        - Get current process ID (returned in x0)
//   3: yield()         - Voluntarily yield CPU to scheduler
//   4: sleep(ms)       - Sleep for specified milliseconds (NYI, just yields)
#[no_mangle]
pub extern "C" fn kernel_syscall_handler(id: u64, arg0: u64, _arg1: u64) -> u64 {
    match id {
        0 => { // print(ptr, len)
            let ptr = arg0 as *const u8;
            let len = _arg1 as usize;
            // println!("[syscall] print ptr={:#x} len={}", arg0, len); // Debug
            // Validate pointer? Assumed valid for now (Shared address space)
            let s = unsafe { 
                let slice = core::slice::from_raw_parts(ptr, len);
                core::str::from_utf8(slice).unwrap_or("<?>")
            };
            print!("{}", s);
            0 // Success
        },
        1 => { // exit()
            sched::exit_current_task();
        },
        2 => { // getpid()
            sched::current_task_id() as u64
        },
        3 => { // yield()
            sched::schedule();
            0
        },
        4 => { // sleep(ms) - placeholder, just yields for now
            // TODO: Implement proper timer-based sleep
            sched::schedule();
            0
        },
        _ => {
            println!("[syscall] Unknown syscall: {}", id);
            u64::MAX // Error
        }
    }
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
