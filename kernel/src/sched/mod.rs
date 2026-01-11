// =============================================================================
// APRK OS - Process Scheduler
// =============================================================================
// Preemptive round-robin scheduler with priority support.
// Uses fixed-size arrays for stability during interrupt context.
// =============================================================================

/// Maximum number of tasks supported
const MAX_TASKS: usize = 16;

/// Scheduler time slice in ticks (higher priority = more slices)
const BASE_TIME_SLICE: usize = 1;

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
#[allow(dead_code)]
#[repr(u8)]
pub enum Priority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    RealTime = 4,
}

impl Priority {
    /// Get time slice multiplier for this priority
    pub fn time_slices(&self) -> usize {
        match self {
            Priority::Idle => 1,
            Priority::Low => 2,
            Priority::Normal => 4,
            Priority::High => 8,
            Priority::RealTime => 16,
        }
    }
}

/// Process Control Block (PCB)
#[repr(C)]
pub struct Task {
    pub id: usize,              // Task ID (PID)
    pub stack_top: u64,         // Saved stack pointer
    pub state: TaskState,       // Current state
    pub priority: Priority,     // Scheduling priority
    pub remaining_slices: usize, // Time slices remaining before preemption
    pub name: [u8; 16],         // Task name (fixed size for safety)
}

impl Task {
    const fn empty() -> Self {
        Task {
            id: 0,
            stack_top: 0,
            state: TaskState::Unused,
            priority: Priority::Idle,
            remaining_slices: 0,
            name: [0u8; 16],
        }
    }
    
    fn set_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = core::cmp::min(bytes.len(), 15);
        self.name[..len].copy_from_slice(&bytes[..len]);
        self.name[len] = 0;
    }
    
    pub fn get_name(&self) -> &str {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(16);
        core::str::from_utf8(&self.name[..len]).unwrap_or("?")
    }
    
    fn reset_time_slice(&mut self) {
        self.remaining_slices = self.priority.time_slices() * BASE_TIME_SLICE;
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
static mut SCHEDULER_ENABLED: bool = false;

/// Initialize the scheduler
pub fn init() {
    unsafe {
        TASKS[0] = Task {
            id: 0,
            stack_top: 0,
            state: TaskState::Running,
            priority: Priority::Idle,
            remaining_slices: 1,
            name: *b"idle\0\0\0\0\0\0\0\0\0\0\0\0",
        };
        TASK_COUNT = 1;
        NEXT_PID = 1;
        SCHEDULER_ENABLED = false;
    }
}

/// Enable preemptive scheduling (call after initial setup)
pub fn enable() {
    unsafe { SCHEDULER_ENABLED = true; }
}

/// Check if scheduler is enabled
#[allow(dead_code)]
pub fn is_enabled() -> bool {
    unsafe { SCHEDULER_ENABLED }
}

/// Spawn a new task with default priority
#[allow(dead_code)]
pub fn spawn(entry: extern "C" fn()) {
    spawn_named(entry, "task", Priority::Normal);
}

/// Spawn a new task with a name and priority (Kernel Thread)
pub fn spawn_named(entry: extern "C" fn(), name: &str, priority: Priority) {
    unsafe {
        if TASK_COUNT >= MAX_TASKS {
            crate::println!("[sched] ERROR: Max tasks ({}) reached!", MAX_TASKS);
            return;
        }
        
        let slot = TASK_COUNT;
        let id = NEXT_PID;
        NEXT_PID += 1;
        
        // Allocate 16KB kernel stack
        let stack_layout = core::alloc::Layout::from_size_align(16 * 1024, 16).unwrap();
        let stack_ptr = alloc::alloc::alloc(stack_layout);
        let mut stack_top = stack_ptr.add(16 * 1024) as u64;
        
        // Setup initial context on stack (Sync with context.S: 112 bytes = 14 u64s)
        let sp = (stack_top as *mut u64).sub(14);
        
        // x19 = entry point (will be read by trampoline)
        *sp.add(0) = entry as u64;
        
        // Context Frame Layout:
        // 0,1: x19,x20
        // ...
        // 10,11: x29,x30
        // 12: SP_EL0
        
        // x30 = return address = trampoline
        *sp.add(11) = task_trampoline as *const () as u64;
        
        // SP_EL0 = 0 (Unused for kernel threads)
        *sp.add(12) = 0;
        
        stack_top = sp as u64;
        
        TASKS[slot].id = id;
        TASKS[slot].stack_top = stack_top;
        TASKS[slot].state = TaskState::Ready;
        TASKS[slot].priority = priority;
        TASKS[slot].set_name(name);
        TASKS[slot].reset_time_slice();
        
        TASK_COUNT += 1;
        
        crate::println!("[sched] Task {} '{}' spawned (priority: {:?})", id, name, priority);
    }
}

/// Spawn a new User Task (EL0)
pub fn spawn_user(entry_addr: u64, name: &str) {
    unsafe {
        if TASK_COUNT >= MAX_TASKS {
            crate::println!("[sched] ERROR: Max tasks reached!");
            return;
        }

        let slot = TASK_COUNT;
        let id = NEXT_PID;
        NEXT_PID += 1;

        // 1. Allocate Kernel Stack (16KB)
        let kstack_layout = core::alloc::Layout::from_size_align(16 * 1024, 16).unwrap();
        let kstack_ptr = alloc::alloc::alloc(kstack_layout);
        let mut kstack_top = kstack_ptr.add(16 * 1024) as u64;

        // 2. Allocate User Stack (64KB, EL0 Accessible)
        // Access permissions handled by paging (Heap is EL0 RW)
        let ustack_layout = core::alloc::Layout::from_size_align(64 * 1024, 16).unwrap();
        let ustack_ptr = alloc::alloc::alloc(ustack_layout);
        // Zero the stack (security/debug)
        core::ptr::write_bytes(ustack_ptr, 0, 64 * 1024);
        let ustack_top = ustack_ptr.add(64 * 1024) as u64;

        // 3. Setup Context on Kernel Stack (112 bytes)
        let sp = (kstack_top as *mut u64).sub(14);

        // x19 = User Entry Point
        *sp.add(0) = entry_addr;
        // x20 = User Stack Pointer
        *sp.add(1) = ustack_top;
        
        // x30 = Return Address = User Trampoline
        *sp.add(11) = user_trampoline as *const () as u64;

        // SP_EL0 = User Stack Pointer (Restored by context_switch)
        *sp.add(12) = ustack_top;

        kstack_top = sp as u64;

        TASKS[slot].id = id;
        TASKS[slot].stack_top = kstack_top;
        TASKS[slot].state = TaskState::Ready;
        TASKS[slot].priority = Priority::Normal; // Default user priority
        TASKS[slot].set_name(name);
        TASKS[slot].reset_time_slice();

        TASK_COUNT += 1;
        crate::println!("[sched] User Task {} '{}' spawned.", id, name);
    }
}

/// Trampoline for new tasks - enables interrupts then jumps to the real entry
#[no_mangle]
extern "C" fn task_trampoline() {
    let entry: extern "C" fn();
    unsafe {
        core::arch::asm!("mov {}, x19", out(reg) entry);
        aprk_arch_arm64::cpu::enable_interrupts();
    }
    // Call the actual entry point
    entry();
    // If entry returns, exit the task
    exit_current_task();
}

/// Trampoline for User Tasks
#[no_mangle]
#[allow(unreachable_code)]
extern "C" fn user_trampoline() {
    let entry: u64;
    let stack: u64;
    unsafe {
        // Load arguments from saved context (regs restored by context_switch)
        core::arch::asm!("mov {}, x19", out(reg) entry);
        core::arch::asm!("mov {}, x20", out(reg) stack);
        
        crate::println!("[sched] Dropping to User Mode: Entry={:#x}, Stack={:#x}", entry, stack);

        // Enable interupts? 
        // enter_user_mode will mask them first, then eret will unmask (via SPSR).
        // For now, we can enable here briefly if needed, but enter_user_mode handles logic.
        
        aprk_arch_arm64::context::enter_user_mode(entry, stack);
    }
    // Should never return
    panic!("User task returned from enter_user_mode!");
}

/// Terminate the current task and switch to another
pub fn exit_current_task() -> ! {
    unsafe {
        let id = TASKS[CURRENT_TASK].id;
        let name = TASKS[CURRENT_TASK].get_name();
        crate::println!("[sched] Task {} '{}' exited.", id, name);
        TASKS[CURRENT_TASK].state = TaskState::Dead;
        schedule();
        loop { aprk_arch_arm64::cpu::halt(); }
    }
}

/// Get the current task ID
pub fn current_task_id() -> usize {
    unsafe { TASKS[CURRENT_TASK].id }
}

/// Print all active tasks
pub fn print_tasks() {
    unsafe {
        crate::println!("PID  STATE     PRIORITY  NAME");
        crate::println!("---  -----     --------  ----");
        for i in 0..TASK_COUNT {
            let task = &TASKS[i];
            crate::println!("{: <3}  {: <9?} {: <9?} {}", 
                task.id, task.state, task.priority, task.get_name());
        }
    }
}

/// Get the number of active tasks
#[allow(dead_code)]
pub fn task_count() -> usize {
    unsafe { TASK_COUNT }
}

/// Block the current task (e.g., waiting for I/O)
#[allow(dead_code)]
pub fn block_current_task() {
    unsafe {
        TASKS[CURRENT_TASK].state = TaskState::Blocked;
        schedule();
    }
}

/// Wake up a blocked task by ID
#[allow(dead_code)]
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

/// Called by timer interrupt - handles time slice decrement
pub fn tick() {
    unsafe {
        // Don't schedule if disabled or only 1 task
        if !SCHEDULER_ENABLED || TASK_COUNT <= 1 {
            return;
        }
        

        
        // Decrement time slice for current task
        if TASKS[CURRENT_TASK].remaining_slices > 0 {
            TASKS[CURRENT_TASK].remaining_slices -= 1;
        }
        
        // Only preempt if time slice expired
        if TASKS[CURRENT_TASK].remaining_slices == 0 {
            schedule();
        }
    }
}

/// Priority-aware round-robin scheduler
pub fn schedule() {
    unsafe {
        let count = TASK_COUNT;
        if count <= 1 || !SCHEDULER_ENABLED { return; }
        
        let current_idx = CURRENT_TASK;
        
        // Find next runnable task with priority consideration
        // Skip task 0 (idle) unless it has a valid stack (we've switched away from it before)
        let mut best_idx = current_idx;
        let mut best_priority = Priority::Idle;
        let mut found = false;
        
        for i in 1..=count {
            let check_idx = (current_idx + i) % count;
            
            // Skip idle task if it hasn't been initialized
            // (stack_top is 0 until we context switch away from it)
            if check_idx == 0 && TASKS[0].stack_top == 0 {
                continue;
            }
            
            let state = TASKS[check_idx].state;
            let priority = TASKS[check_idx].priority;
            
            if state == TaskState::Ready {
                if !found || priority > best_priority {
                    best_idx = check_idx;
                    best_priority = priority;
                    found = true;
                }
            }
        }
        
        // If no ready task found, check if we should stay on current
        if !found {
            let current_state = TASKS[current_idx].state;
            if current_state == TaskState::Running {
                // Current task still runnable, keep running
                TASKS[current_idx].reset_time_slice();
                return;
            } else if current_state == TaskState::Dead || current_state == TaskState::Blocked {
                // Try to switch to idle
                if TASKS[0].stack_top != 0 {
                    TASKS[0].state = TaskState::Running;
                    CURRENT_TASK = 0;
                    let prev_sp = &mut TASKS[current_idx].stack_top as *mut u64;
                    let next_sp = TASKS[0].stack_top;
                    aprk_arch_arm64::context::context_switch(prev_sp, next_sp);
                }
                // If idle isn't ready either, halt
                crate::println!("[sched] FATAL: No runnable tasks!");
                loop { aprk_arch_arm64::cpu::halt(); }
            }
            return;
        }
        
        // Don't switch to self
        if best_idx == current_idx {
            TASKS[current_idx].reset_time_slice();
            return;
        }
        
        // Mark old task as Ready (if it was Running)
        if TASKS[current_idx].state == TaskState::Running {
            TASKS[current_idx].state = TaskState::Ready;
        }
        
        // Switch to new task
        TASKS[best_idx].state = TaskState::Running;
        TASKS[best_idx].reset_time_slice();
        CURRENT_TASK = best_idx;
        
        // Perform Context Switch
        let prev_sp = &mut TASKS[current_idx].stack_top as *mut u64;
        let next_sp = TASKS[best_idx].stack_top;
        
        aprk_arch_arm64::context::context_switch(prev_sp, next_sp);
    }
}
