// =============================================================================
// APRK OS - Process Scheduler
// =============================================================================
// Manages tasks and context switching with priority support.
// =============================================================================

use alloc::vec::Vec;
use alloc::string::String;
use aprk_arch_arm64::cpu;

/// Task execution states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,      // Can be scheduled
    Running,    // Currently executing
    Blocked,    // Waiting for I/O or event
    Dead,       // Terminated, awaiting cleanup
}

/// Task priority levels (higher = more important)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    RealTime = 4,
}

/// Process Control Block (PCB)
#[derive(Debug)]
pub struct Task {
    pub id: usize,              // Task ID (PID)
    pub name: String,           // Task name for debugging
    pub stack_top: u64,         // Saved stack pointer
    pub state: TaskState,       // Current state
    pub priority: Priority,     // Scheduling priority
    // Future: page_table: u64, // TTBR0 for this process
}

static mut TASKS: Vec<Task> = Vec::new();
static mut CURRENT_TASK: usize = 0;
static mut NEXT_PID: usize = 0;


pub fn init() {
    // Create the "Idle" task (Task 0), which is just the boot kernel context
    // We don't allocate a stack for it because it's already running on the boot stack.
    let idle = Task {
        id: 0,
        name: String::from("idle"),
        stack_top: 0, // Current SP
        state: TaskState::Running,
        priority: Priority::Idle,
    };
    
    unsafe {
        TASKS = Vec::new();
        TASKS.push(idle);
        NEXT_PID = 1;
    }
}

/// Spawn a new task with default priority
pub fn spawn(entry: extern "C" fn()) {
    spawn_named(entry, "task", Priority::Normal);
}

/// Spawn a new task with a name and priority
pub fn spawn_named(entry: extern "C" fn(), name: &str, priority: Priority) {
    let id = unsafe { 
        let pid = NEXT_PID;
        NEXT_PID += 1;
        pid
    };
    
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
        *sp.add(11) = task_trampoline as *const () as u64;
        
        stack_top = sp as u64;
    }
    
    crate::println!("[sched] Spawning Task {} '{}' (Entry: {:#x}, Priority: {:?})", 
                    id, name, entry as u64, priority);
    
    let task = Task {
        id,
        name: String::from(name),
        stack_top,
        state: TaskState::Ready,
        priority,
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
        let task = &TASKS[CURRENT_TASK];
        crate::println!("[sched] Task {} '{}' exiting.", task.id, task.name);
        TASKS[CURRENT_TASK].state = TaskState::Dead;
        schedule();
        loop { aprk_arch_arm64::cpu::halt(); }
    }
}

/// Get the current task ID
pub fn current_task_id() -> usize {
    unsafe { TASKS[CURRENT_TASK].id }
}

/// Get the number of tasks
pub fn task_count() -> usize {
    unsafe { TASKS.len() }
}

/// Block the current task (e.g., waiting for I/O)
pub fn block_current_task() {
    unsafe {
        TASKS[CURRENT_TASK].state = TaskState::Blocked;
        schedule();
    }
}

/// Wake up a blocked task by ID
pub fn wake_task(pid: usize) {
    unsafe {
        for task in TASKS.iter_mut() {
            if task.id == pid && task.state == TaskState::Blocked {
                task.state = TaskState::Ready;
                return;
            }
        }
    }
}

/// Round-robin scheduler with priority awareness
pub fn schedule() {
    unsafe {
        if TASKS.len() <= 1 { return; }
        
        let current_idx = CURRENT_TASK;
        let num_tasks = TASKS.len();
        
        // Find next runnable task (Ready state, not Dead or Blocked)
        let mut next_idx = (current_idx + 1) % num_tasks;
        let mut found = false;
        
        for _ in 0..num_tasks {
            if next_idx != current_idx {
                let state = TASKS[next_idx].state;
                if state == TaskState::Ready || state == TaskState::Running {
                    found = true;
                    break;
                }
            }
            next_idx = (next_idx + 1) % num_tasks;
        }
        
        if !found {
            // Check if current task is still runnable
            let current_state = TASKS[current_idx].state;
            if current_state == TaskState::Dead || current_state == TaskState::Blocked {
                // No runnable tasks and current is dead/blocked
                crate::println!("[sched] No runnable tasks! Halting.");
                loop { aprk_arch_arm64::cpu::halt(); }
            }
            return; // Stay on current task
        }
        
        // Mark old task as Ready (if it was Running)
        if TASKS[current_idx].state == TaskState::Running {
            TASKS[current_idx].state = TaskState::Ready;
        }
        
        // Switch to new task
        TASKS[next_idx].state = TaskState::Running;
        CURRENT_TASK = next_idx;
        
        // Debug: context switch (commented to reduce noise)
        // crate::println!("[sched] Switch: {} -> {} ('{}')", 
        //                 current_idx, next_idx, TASKS[next_idx].name);
        
        // Perform Context Switch
        let prev_sp = &mut TASKS[current_idx].stack_top as *mut u64;
        let next_sp = TASKS[next_idx].stack_top;
        
        aprk_arch_arm64::context::context_switch(prev_sp, next_sp);
    }
}
