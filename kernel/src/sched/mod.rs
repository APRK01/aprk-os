// =============================================================================
// APRK OS - Process Scheduler (Round Robin)
// =============================================================================
// Manages tasks and context switching.
// =============================================================================

use alloc::vec::Vec;
use aprk_arch_arm64::cpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Dead,
}

#[derive(Debug)]
pub struct Task {
    pub id: usize,
    pub stack_top:  u64,
    pub state: TaskState,
}

static mut TASKS: Vec<Task> = Vec::new();
static mut CURRENT_TASK: usize = 0;

pub fn init() {
    // Create the "Idle" task (Task 0), which is just the boot kernel context
    // We don't allocate a stack for it because it's already running on the boot stack.
    let idle = Task {
        id: 0,
        stack_top: 0, // Current SP
        state: TaskState::Running,
    };
    
    unsafe {
        TASKS = Vec::new();
        TASKS.push(idle);
    }
}

pub fn spawn(entry: extern "C" fn()) {
    let id = unsafe { TASKS.len() };
    
    // Allocate 16KB stack
    let stack_layout = core::alloc::Layout::from_size_align(16 * 1024, 16).unwrap();
    let stack_ptr = unsafe { alloc::alloc::alloc(stack_layout) };
    let mut stack_top = unsafe { stack_ptr.add(16 * 1024) as u64 };
    
    // Setup initial context on stack
    // We need space for 12 callee-saved registers (x19-x30)
    // We'll use x19 to pass the entry point to our trampoline
    unsafe {
        let sp = (stack_top as *mut u64).sub(12);
        
        // x19 = entry point (will be read by trampoline)
        *sp.add(0) = entry as u64;
        // x30 = return address = trampoline
        *sp.add(11) = task_trampoline as u64;
        
        stack_top = sp as u64;
    }
    
    crate::println!("[sched] Spawning Task {} (Entry: {:#x}, Stack: {:#x})", id, entry as u64, stack_top);
    
    let task = Task {
        id,
        stack_top,
        state: TaskState::Ready,
    };
    
    unsafe { TASKS.push(task) };
}

/// Trampoline for new tasks - enables interrupts then jumps to the real entry
#[no_mangle]
extern "C" fn task_trampoline() {
    // x19 contains the real entry point (set up by spawn)
    let entry: extern "C" fn();
    unsafe {
        core::arch::asm!("mov {}, x19", out(reg) entry);
    }
    
    // Enable interrupts for this task
    unsafe { aprk_arch_arm64::cpu::enable_interrupts(); }
    
    // Call the actual entry point
    entry();
    
    // If entry returns, exit the task
    exit_current_task();
}

/// Terminate the current task and switch to another
pub fn exit_current_task() -> ! {
    unsafe {
        crate::println!("[sched] Task {} Exiting.", TASKS[CURRENT_TASK].id);
        TASKS[CURRENT_TASK].state = TaskState::Dead;
        schedule();
        loop { aprk_arch_arm64::cpu::halt(); }
    }
}

pub fn schedule() {
    unsafe {
        if TASKS.len() <= 1 { return; }
        
        let current_id = CURRENT_TASK;
        let mut next_id = (current_id + 1) % TASKS.len();
        
        // Find next non-dead task
        loop {
            if next_id == current_id {
                // We wrapped around.
                if TASKS[current_id].state == TaskState::Dead {
                    // Everyone is dead! Panic.
                    crate::println!("[sched] All tasks dead! Halting.");
                    loop { aprk_arch_arm64::cpu::halt(); }
                }
                return; // Nothing new to run, stay on current (if alive).
            }

            if TASKS[next_id].state != TaskState::Dead {
                break; // Found one
            }
            next_id = (next_id + 1) % TASKS.len();
        }
        
        CURRENT_TASK = next_id;
        
        crate::println!("[sched] Switch: {} -> {}", current_id, next_id);
        
        // Perform Context Switch
        let prev_sp = &mut TASKS[current_id].stack_top as *mut u64;
        let next_sp = TASKS[next_id].stack_top;
        
        aprk_arch_arm64::context::context_switch(prev_sp, next_sp);
    }
}
