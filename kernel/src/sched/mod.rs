// =============================================================================
// APRK OS - Process Scheduler
// =============================================================================
// Manages tasks and context switching with priority support.
// Uses fixed-size arrays for stability during interrupt context.
// =============================================================================

use alloc::string::String;

/// Maximum number of tasks supported
const MAX_TASKS: usize = 16;

/// Task execution states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Unused,     // Slot is available
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
#[repr(C)]
pub struct Task {
    pub id: usize,              // Task ID (PID)
    pub stack_top: u64,         // Saved stack pointer
    pub state: TaskState,       // Current state
    pub priority: Priority,     // Scheduling priority
    pub name: [u8; 16],         // Task name (fixed size for safety)
}

impl Task {
    const fn empty() -> Self {
        Task {
            id: 0,
            stack_top: 0,
            state: TaskState::Unused,
            priority: Priority::Idle,
            name: [0u8; 16],
        }
    }
    
    fn set_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = core::cmp::min(bytes.len(), 15);
        self.name[..len].copy_from_slice(&bytes[..len]);
        self.name[len] = 0;
    }
    
    fn get_name(&self) -> &str {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(16);
        core::str::from_utf8(&self.name[..len]).unwrap_or("?")
    }
}

// Fixed-size task array - no heap allocation during access
static mut TASKS: [Task; MAX_TASKS] = [
    Task::empty(), Task::empty(), Task::empty(), Task::empty(),
    Task::empty(), Task::empty(), Task::empty(), Task::empty(),
    Task::empty(), Task::empty(), Task::empty(), Task::empty(),
    Task::empty(), Task::empty(), Task::empty(), Task::empty(),
];

static mut TASK_COUNT: usize = 0;
static mut CURRENT_TASK: usize = 0;
static mut NEXT_PID: usize = 0;

pub fn init() {
    // Create the "Idle" task (Task 0), which is just the boot kernel context
    unsafe {
        TASKS[0] = Task {
            id: 0,
            stack_top: 0,
            state: TaskState::Running,
            priority: Priority::Idle,
            name: *b"idle\0\0\0\0\0\0\0\0\0\0\0\0",
        };
        TASK_COUNT = 1;
        NEXT_PID = 1;
    }
}

/// Spawn a new task with default priority
pub fn spawn(entry: extern "C" fn()) {
    spawn_named(entry, "task", Priority::Normal);
}

/// Spawn a new task with a name and priority
pub fn spawn_named(entry: extern "C" fn(), name: &str, priority: Priority) {
    unsafe {
        if TASK_COUNT >= MAX_TASKS {
            crate::println!("[sched] ERROR: Max tasks reached!");
            return;
        }
        
        let slot = TASK_COUNT;
        let id = NEXT_PID;
        NEXT_PID += 1;
        
        // Allocate 16KB stack
        let stack_layout = core::alloc::Layout::from_size_align(16 * 1024, 16).unwrap();
        let stack_ptr = alloc::alloc::alloc(stack_layout);
        let mut stack_top = stack_ptr.add(16 * 1024) as u64;
        
        // Setup initial context on stack
        // We need space for 12 callee-saved registers (x19-x30)
        // We'll use x19 to pass the entry point to our trampoline
        let sp = (stack_top as *mut u64).sub(12);
        
        // x19 = entry point (will be read by trampoline)
        *sp.add(0) = entry as u64;
        // x30 = return address = trampoline
        *sp.add(11) = task_trampoline as *const () as u64;
        
        stack_top = sp as u64;
        
        crate::println!("[sched] Spawning Task {} '{}' (Entry: {:#x}, Priority: {:?})", 
                        id, name, entry as u64, priority);
        
        TASKS[slot].id = id;
        TASKS[slot].stack_top = stack_top;
        TASKS[slot].state = TaskState::Ready;
        TASKS[slot].priority = priority;
        TASKS[slot].set_name(name);
        
        TASK_COUNT += 1;
    }
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
        crate::println!("[sched] Task {} '{}' exiting.", task.id, task.get_name());
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
    unsafe { TASK_COUNT }
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
        for i in 0..TASK_COUNT {
            if TASKS[i].id == pid && TASKS[i].state == TaskState::Blocked {
                TASKS[i].state = TaskState::Ready;
                return;
            }
        }
    }
}

/// Round-robin scheduler
pub fn schedule() {
    unsafe {
        let count = TASK_COUNT;
        if count <= 1 { return; }
        
        let current_idx = CURRENT_TASK;
        
        // Find next runnable task (Ready state, not Dead or Blocked)
        let mut next_idx = current_idx;
        let mut found = false;
        
        for i in 1..=count {
            let check_idx = (current_idx + i) % count;
            let state = TASKS[check_idx].state;
            if state == TaskState::Ready || (check_idx != current_idx && state == TaskState::Running) {
                next_idx = check_idx;
                found = true;
                break;
            }
        }
        
        if !found {
            // No other runnable tasks - stay on current if it's runnable
            let current_state = TASKS[current_idx].state;
            if current_state == TaskState::Dead || current_state == TaskState::Blocked {
                crate::println!("[sched] No runnable tasks! Halting.");
                loop { aprk_arch_arm64::cpu::halt(); }
            }
            return;
        }
        
        // Don't switch to self
        if next_idx == current_idx {
            return;
        }
        
        // Mark old task as Ready (if it was Running)
        if TASKS[current_idx].state == TaskState::Running {
            TASKS[current_idx].state = TaskState::Ready;
        }
        
        // Switch to new task
        TASKS[next_idx].state = TaskState::Running;
        CURRENT_TASK = next_idx;
        
        // Perform Context Switch
        let prev_sp = &mut TASKS[current_idx].stack_top as *mut u64;
        let next_sp = TASKS[next_idx].stack_top;
        
        aprk_arch_arm64::context::context_switch(prev_sp, next_sp);
    }
}
