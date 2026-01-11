use aprk_arch_arm64::{print, println};
use crate::sched;

pub fn handle_syscall(id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    match id {
        0 => { // print(ptr, len)
            let ptr = arg0 as *const u8;
            let len = arg1 as usize;
            if !ptr.is_null() && len > 0 {
                let s = unsafe { 
                    let slice = core::slice::from_raw_parts(ptr, len);
                    core::str::from_utf8(slice).unwrap_or("<?>")
                };
                print!("{}", s);
            }
            0
        },
        1 => { // exit()
            sched::exit_current_task();
            0
        },
        2 => { // getpid()
            sched::current_task_id() as u64
        },
        3 => { // yield()
            sched::schedule();
            0
        },
        4 => { // sleep(ms)
            // Placeholder: yield for now
            sched::schedule();
            0
        },
        5 => { // alloc(size, align)
            let size = arg0 as usize;
            let align = arg1 as usize;
            match core::alloc::Layout::from_size_align(size, align) {
                Ok(layout) => {
                    let ptr = unsafe { alloc::alloc::alloc(layout) as u64 };
                    if ptr == 0 { 0 } else { ptr }
                },
                Err(_) => 0,
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
                1
            }
        },
        _ => {
            println!("[syscall] Unknown syscall: {}", id);
            u64::MAX
        }
    }
}
