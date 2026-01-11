use virtio_drivers::{BufferDirection, Hal, PhysAddr};
use core::ptr::NonNull;
use alloc::alloc::{alloc, dealloc, Layout};

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(pages * 4096, 4096).unwrap();
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!("VirtIO HAL: Failed to allocate DMA memory");
        }
        (ptr as usize, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(phys: PhysAddr, _virt: NonNull<u8>, pages: usize) -> i32 {
        let layout = Layout::from_size_align(pages * 4096, 4096).unwrap();
        dealloc(phys as *mut u8, layout);
        0
    }

    #[allow(unused_variables)]
    unsafe fn mmio_phys_to_virt(phys: PhysAddr, size: usize) -> NonNull<u8> {
        NonNull::new(phys as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        buffer.as_ptr() as *mut u8 as usize
    }

    unsafe fn unshare(_phys: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}



pub fn init() {
    // Discovery logic will be handled by specific drivers or a general bus scan later.
    // For now, GPU driver will use its own discovery at a known MMIO address.
}
