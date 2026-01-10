pub mod pmm;
pub mod heap;

pub fn init() {
    // We need the end of the kernel to know where free memory starts.
    // This symbol comes from the linker script.
    extern "C" {
        static __kernel_end: usize;
    }
    
    let kernel_end = unsafe { &__kernel_end as *const _ as usize };
    
    pmm::init(kernel_end);
    heap::init();
}
