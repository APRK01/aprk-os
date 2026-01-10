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
    let stack_ptr_u64 = stack_top as *mut u64;
    unsafe {
        // We need to write 12 registers (x19..x30)
        // x30 (LR) is at index 11 (offset 88 bytes)
        // Decrement SP by 96 bytes first
        let sp = stack_ptr_u64.sub(12);
        
        // Write Entry point to LR (x30)
        // sp[11] = entry address
        *sp.add(11) = entry as u64;
        
        // Update stack_top to point to the new SP
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

pub fn schedule() {
    unsafe {
        if TASKS.len() <= 1 { return; }
        
        let current_id = CURRENT_TASK;
        let next_id = (current_id + 1) % TASKS.len();
        
        if current_id == next_id { return; }
        
        CURRENT_TASK = next_id;
        
        crate::println!("[sched] Switch: {} -> {}", current_id, next_id);
        
        // Perform Context Switch
        let prev_sp = &mut TASKS[current_id].stack_top as *mut u64;
        let next_sp = TASKS[next_id].stack_top;
        
        aprk_arch_arm64::context::context_switch(prev_sp, next_sp);
    }
}
