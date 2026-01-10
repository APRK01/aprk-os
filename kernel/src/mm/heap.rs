// =============================================================================
// APRK OS - Heap Allocator
// =============================================================================
// Initializes the Global Allocator so we can use Box, Vec, String, etc.
// Uses linked_list_allocator crate for stability.
// =============================================================================

use linked_list_allocator::LockedHeap;
use super::pmm;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Heap starts after the kernel bitmap, let's pick a safe spot.
// RAM: 0x4000_0000 
// Kernel loads at 0x4008_0000.
// Let's put the Heap at 0x4100_0000 (16MB mark) and give it 16MB.
pub const HEAP_START: usize = 0x4100_0000;
pub const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16 MB

pub fn init() {
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }
    crate::println!("[mm] Heap Initialized at {:#x} (Size: {} MB)", HEAP_START, HEAP_SIZE / 1024 / 1024);
}

// Handler for Allocation Errors (OOM)
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
