#![no_std]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;

// =============================================================================
// APRK OS - Userspace Library
// =============================================================================
// System call wrappers for user programs.
// =============================================================================

/// Print a string to the console.
/// Syscall 0: print(ptr, len)
pub fn print(s: &str) {
    unsafe {
        core::arch::asm!(
            "mov x8, #0", // Syscall ID: PRINT
            "svc #0",
            in("x0") s.as_ptr(),
            in("x1") s.len(),
            clobber_abi("C")
        );
    }
}

/// Exit the current process.
/// Syscall 1: exit()
pub fn exit() -> ! {
    unsafe {
        core::arch::asm!(
            "mov x8, #1", // Syscall ID: EXIT
            "svc #0",
            options(noreturn)
        );
    }
}

/// Get the current process ID.
/// Syscall 2: getpid() -> pid
pub fn getpid() -> u64 {
    let pid: u64;
    unsafe {
        core::arch::asm!(
            "mov x8, #2", // Syscall ID: GETPID
            "svc #0",
            out("x0") pid,
            clobber_abi("C")
        );
    }
    pid
}

/// Voluntarily yield the CPU to the scheduler.
/// Syscall 3: yield()
pub fn yield_cpu() {
    unsafe {
        core::arch::asm!(
            "mov x8, #3", // Syscall ID: YIELD
            "svc #0",
            clobber_abi("C")
        );
    }
}

/// Sleep for the specified number of milliseconds.
/// Syscall 4: sleep(ms)
/// Note: Currently just yields, proper timing not yet implemented.
pub fn sleep(_ms: u64) {
    unsafe {
        core::arch::asm!(
            "mov x8, #4", // Syscall ID: SLEEP
            "svc #0",
            in("x0") _ms,
            clobber_abi("C")
        );
    }
}

// Convenience macros for printing
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut w = $crate::PrintWriter;
        let _ = write!(w, $($arg)*);
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Writer for the print macros
pub struct PrintWriter;

impl core::fmt::Write for PrintWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print(s);
        Ok(())
    }
}

// Allocator implementation
use core::alloc::{GlobalAlloc, Layout};

pub struct UserAllocator;

unsafe impl GlobalAlloc for UserAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let ptr: *mut u8;
        core::arch::asm!(
            "mov x0, {size}",
            "mov x1, {align}",
            "mov x8, #5", // Syscall ID: ALLOC
            "svc #0",
            // We use latout("x0") to capture the return value directly
            // from x0, which is preserved by clobber_abi("C") for return usage?
            // Actually, clobber_abi marks x0 as clobbered output.
            // So we just take it from x0 as an output.
            size = in(reg) size,
            align = in(reg) align,
            lateout("x0") ptr,
            clobber_abi("C")
        );
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        let align = layout.align();
        core::arch::asm!(
            "mov x0, {ptr}",
            "mov x1, {size}",
            "mov x2, {align}",
            "mov x8, #6", // Syscall ID: DEALLOC
            "svc #0",
            ptr = in(reg) ptr,
            size = in(reg) size,
            align = in(reg) align,
            clobber_abi("C")
        );
    }
}

#[global_allocator]
static ALLOCATOR: UserAllocator = UserAllocator;

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    print("User Allocation Error: Out of Memory\n");
    exit();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print("PANIC in user mode: ");
    if let Some(location) = info.location() {
        print(location.file());
        print(":");
        // Can't easily print numbers without alloc, so just show the message
    }
    print("\n");
    loop { unsafe { core::arch::asm!("wfe") }; }
}

