// =============================================================================
// APRK OS - Physical Memory Manager (PMM)
// =============================================================================
// Tracks usage of physical RAM using a bitmap.
// =============================================================================

use core::sync::atomic::{AtomicUsize, Ordering};

// Memory Map for QEMU Virt
pub const RAM_START: usize = 0x4000_0000;
pub const RAM_SIZE: usize = 512 * 1024 * 1024; // 512 MB
pub const PAGE_SIZE: usize = 4096;
pub const TOTAL_PAGES: usize = RAM_SIZE / PAGE_SIZE; // 131,072 pages

// Bitmap size: 1 bit per page.
// 131,072 bits / 64 bits/u64 = 2048 u64s = 16KB
const BITMAP_SIZE: usize = TOTAL_PAGES / 64;

static mut BITMAP: [u64; BITMAP_SIZE] = [0; BITMAP_SIZE];
static ALLOC_START: AtomicUsize = AtomicUsize::new(0);

/// Initialize the PMM.
/// Marks kernel memory as used.
pub fn init(kernel_end: usize) {
    let kernel_pages = (kernel_end - RAM_START + PAGE_SIZE - 1) / PAGE_SIZE;
    
    // Mark kernel pages as used
    for i in 0..kernel_pages {
        unsafe { set_bit(i) };
    }
    
    // Set search start hint
    ALLOC_START.store(kernel_pages, Ordering::Relaxed);
    
    crate::println!("[mm] PMM Initialized. Kernel uses {} pages.", kernel_pages);
}

/// Allocate a single physical page.
/// Returns the physical address.
#[allow(dead_code)]
pub fn alloc_page() -> Option<usize> {
    let start = ALLOC_START.load(Ordering::Relaxed);
    
    for i in start..TOTAL_PAGES {
        if unsafe { !is_bit_set(i) } {
            unsafe { set_bit(i) };
            ALLOC_START.store(i + 1, Ordering::Relaxed);
            return Some(RAM_START + i * PAGE_SIZE);
        }
    }
    
    // Wrap around if needed (primitive)
    None
}

/// Free a physical page.
#[allow(dead_code)]
pub fn free_page(phys_addr: usize) {
    if phys_addr < RAM_START || phys_addr >= RAM_START + RAM_SIZE {
        return;
    }
    
    let page_idx = (phys_addr - RAM_START) / PAGE_SIZE;
    unsafe { clear_bit(page_idx) };
    
    // Reset hint if we freed a lower page
    let current_start = ALLOC_START.load(Ordering::Relaxed);
    if page_idx < current_start {
        ALLOC_START.store(page_idx, Ordering::Relaxed);
    }
}

// Bitmap Helpers
unsafe fn set_bit(idx: usize) {
    BITMAP[idx / 64] |= 1 << (idx % 64);
}

#[allow(dead_code)]
unsafe fn clear_bit(idx: usize) {
    BITMAP[idx / 64] &= !(1 << (idx % 64));
}

#[allow(dead_code)]
unsafe fn is_bit_set(idx: usize) -> bool {
    (BITMAP[idx / 64] & (1 << (idx % 64))) != 0
}
