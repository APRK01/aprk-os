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
    
    println!("[kernel] Booted. Current EL: {}", arch::cpu::current_el());
    
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
    sched::spawn_named(shell::run, "shell", sched::Priority::Normal);
    // sched::spawn(task_two);
    
    // Test Heap
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    println!("[kernel] Heap Test: Vec = {:?}", v);
    
    // Enable preemptive scheduling now that everything is set up
    sched::enable();
    println!("[kernel] Preemptive scheduler enabled.");
    
    // Switch to the shell immediately
    sched::schedule();
    
    println!();
    println!("[kernel] System ready. (Press Ctrl+A, X to exit QEMU)");

    // Enter main loop - we just wait for interrupts now
    loop {
        // Wait For Event - puts CPU to sleep until interrupt fires
        unsafe { core::arch::asm!("wfe"); }
    }
}


// Timer Callback - called by IRQ handler
#[no_mangle]
pub extern "Rust" fn kernel_tick() {
    sched::tick();
}

// Syscall Handler
// Syscall Numbers:
//   0: print(ptr, len) - Print string to console
//   1: exit()          - Terminate current process
//   2: getpid()        - Get current process ID (returned in x0)
//   3: yield()         - Voluntarily yield CPU to scheduler
//   4: sleep(ms)       - Sleep for specified milliseconds
//   5: alloc(size, align) -> ptr
//   6: dealloc(ptr, size, align)
#[no_mangle]
pub extern "C" fn kernel_syscall_handler(id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    match id {
        0 => { // print(ptr, len)
            let ptr = arg0 as *const u8;
            let len = arg1 as usize;
            
            // println!("[syscall] print(ptr={:#p}, len={})", ptr, len);

            if !ptr.is_null() && len > 0 {
                // Validate pointer? Assumed valid for now (Shared address space)
                // We use slice::from_raw_parts only if check passes
                let s = unsafe { 
                    let slice = core::slice::from_raw_parts(ptr, len);
                    core::str::from_utf8(slice).unwrap_or("<?>")
                };
                print!("{}", s);
            }
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
        5 => { // alloc(size, align)
            let size = arg0 as usize;
            let align = arg1 as usize;
            println!("[syscall] ALLOC(size={}, align={})", size, align);
            
            match core::alloc::Layout::from_size_align(size, align) {
                Ok(layout) => {
                    let ptr = unsafe { alloc::alloc::alloc(layout) as u64 };
                    println!("[syscall] -> Ptr: {:#x}", ptr);
                    if ptr == 0 {
                        println!("[syscall] Kernel Allocator returned NULL!");
                        0
                    } else {
                        ptr
                    }
                },
                Err(e) => {
                     println!("[syscall] Layout Error: {:?}", e);
                     0
                }
            }
        },
        6 => { // dealloc(ptr, size, align)
            let ptr = arg0 as *mut u8;
            let size = arg1 as usize;
            let align = arg2 as usize;
            if let Ok(layout) = core::alloc::Layout::from_size_align(size, align) {
                unsafe { alloc::alloc::dealloc(ptr, layout); }
                0
            } else {
                1 // Error
            }
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
    // Cyan color for logo
    print!("\x1b[1;36m");
    println!(r"    _    ____  ____  _  __   ___  ____  ");
    println!(r"   / \  |  _ \|  _ \| |/ /  / _ \/ ___| ");
    println!(r"  / _ \ | |_) | |_) | ' /  | | | \___ \ ");
    println!(r" / ___ \|  __/|  _ <| . \  | |_| |___) |");
    println!(r"/_/   \_\_|   |_| \_\_|\_\  \___/|____/ ");
    print!("\x1b[0m");
    println!();
    // Yellow for version
    print!("\x1b[1;33m");
    println!("APRK OS v{} \"{}\"", VERSION, CODENAME);
    print!("\x1b[0m");
    println!("A modern operating system kernel for ARM64");
    println!();
    print!("\x1b[90m");
    println!("════════════════════════════════════════════════════════════════");
    print!("\x1b[0m");
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
