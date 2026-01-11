#![no_std]
#![no_main]

// Enable alloc_error_handler feature locally
#![feature(alloc_error_handler)]

extern crate alloc;
use alloc::alloc::{alloc, Layout};
use aprk_user_lib::{print, yield_cpu};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("\n[TEST] Manual Alloc... ");
    
    unsafe {
        let layout = Layout::from_size_align(32, 8).unwrap();
        let ptr = alloc(layout);
        
        if ptr.is_null() {
            print("FAILED (NULL)\n");
        } else {
            print("OK! Ptr valid.\n");
            
            // Try writing (Test WnR)
            *ptr = 0xAA;
            if *ptr == 0xAA {
                print("Write/Read Verified.\n");
            }
        }
    }

    print("Done.\n");
    
    loop {
        yield_cpu();
    }
}


