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
use core::panic::PanicInfo;
use crate::syscall::handle_syscall;

mod drivers;
pub mod fs;
mod loader;
mod mm;
mod sched;
mod shell;
mod syscall;

/// APRK OS version
const VERSION: &str = "0.0.1";

/// APRK OS codename
const CODENAME: &str = "Genesis";

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // 1. Initialize architecture-specific hardware (MMU, Exceptions, GIC, Timer)
    arch::init();
    
    // 2. Initialize Memory Management (PMM + Heap)
    mm::init();
    
    // 3. Initialize Hardware Drivers (GPU, Block)
    drivers::init();
    
    // 20% - HW Ready
    drivers::gpu::update_progress(20);
    
    // Print the APRK OS banner
    print_banner();
    print_system_info();

    // 40% - Banner Displayed
    drivers::gpu::update_progress(40);

    // 4. Initialize FileSystem
    fs::init();
    
    // 60% - FileSystem Ready
    drivers::gpu::update_progress(60);

    // 5. Initialize Scheduler
    sched::init();
    
    // 80% - Scheduler Ready
    drivers::gpu::update_progress(80);

    // 6. Enable Scheduling
    sched::enable();
    println!("[kernel] Preemptive scheduler enabled.");
    
    // 100% - System Ready
    drivers::gpu::update_progress(100);
    println!("[kernel] System ready. (Press Ctrl+A, X to exit QEMU)");

    // 7. Spawn Shell
    sched::spawn_named(shell::shell_task, "shell", sched::Priority::High);

    // 8. Start Scheduling
    sched::schedule();

    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}

#[no_mangle]
pub extern "Rust" fn kernel_tick() {
    sched::tick();
}

#[no_mangle]
pub extern "C" fn kernel_syscall_handler(id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    handle_syscall(id, arg0, arg1, arg2)
}

fn print_banner() {
    println!();
    println!("\x1b[1;36m    _    ____  ____  _  __   ___  ____  \x1b[0m");
    println!("\x1b[1;36m   / \\  |  _ \\|  _ \\| |/ /  / _ \\/ ___| \x1b[0m");
    println!("\x1b[1;36m  / _ \\ | |_) | |_) | ' /  | | | \\___ \\ \x1b[0m");
    println!("\x1b[1;36m / ___ \\|  __/|  _ <| . \\  | |_| |___) |\x1b[0m");
    println!("\x1b[1;36m/_/   \\_\\_|   |_| \\_\\_|_\\_\\  \\___/|____/ \x1b[0m");
    println!();
    println!("\x1b[1;33mAPRK OS v{} \"{}\"\x1b[0m", VERSION, CODENAME);
    println!("A modern operating system kernel for ARM64");
    println!("════════════════════════════════════════════════════════════════");
}

fn print_system_info() {
    println!("[boot] Kernel loaded successfully");
    println!("[boot] Current Exception Level: EL{}", cpu::current_el());
    println!("[boot] Stack Pointer: {:#018x}", cpu::read_sp());
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!();
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!("!!                     KERNEL PANIC                        !!");
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!();
    if let Some(location) = info.location() {
        println!("Location: {}:{}:{}", location.file(), location.line(), location.column());
    }
    println!("Message: {}", info.message());
    println!();
    println!("System halted.");
    cpu::halt();
}
