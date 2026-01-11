#![no_std]
#![no_main]

use aprk_user_lib::{print, exit, getpid, yield_cpu};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("\n");
    print("=========================================\n");
    print("  Hello from USER SPACE! (Phase 10)      \n");
    print("=========================================\n");
    
    // Demonstrate getpid syscall
    let pid = getpid();
    print("My PID: ");
    print_num(pid);
    print("\n");
    
    print("I am a separate ELF binary loaded from TarFS.\n");
    print("I am making System Calls via SVC instruction.\n");
    
    // Demonstrate yield syscall
    print("Yielding CPU...\n");
    yield_cpu();
    print("Back from yield!\n");
    
    print("\nExiting now...\n");
    exit();
}

// Simple number printing without alloc
fn print_num(n: u64) {
    if n == 0 {
        print("0");
        return;
    }
    
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut num = n;
    
    while num > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }
    
    // Print in reverse
    while i > 0 {
        i -= 1;
        let c = buf[i] as char;
        let s: [u8; 1] = [c as u8];
        // Convert single char to str
        if let Ok(s) = core::str::from_utf8(&s) {
            print(s);
        }
    }
}
