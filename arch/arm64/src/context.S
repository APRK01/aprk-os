// =============================================================================
// APRK OS - Context Switch
// =============================================================================
// Switches CPU execution between two tasks.
//
// signature: extern "C" fn context_switch(prev_sp: &mut u64, next_sp: u64);
// x0 = Pointer to where to save current SP
// x1 = Value of next SP
// =============================================================================

.global context_switch

context_switch:
    // 1. Save Callee-Saved Registers (x19-x30) + SP_EL0
    sub     sp, sp, #112
    stp     x19, x20, [sp, #16 * 0]
    stp     x21, x22, [sp, #16 * 1]
    stp     x23, x24, [sp, #16 * 2]
    stp     x25, x26, [sp, #16 * 3]
    stp     x27, x28, [sp, #16 * 4]
    stp     x29, x30, [sp, #16 * 5]  // x29=FP, x30=LR

    // Save SP_EL0
    mrs     x10, sp_el0
    str     x10, [sp, #16 * 6]

    // 2. Save current SP to *x0
    mov     x19, sp
    str     x19, [x0]

    // 3. Load next SP from x1
    mov     sp, x1

    // 4. Restore Callee-Saved Registers + SP_EL0
    ldp     x19, x20, [sp, #16 * 0]
    ldp     x21, x22, [sp, #16 * 1]
    ldp     x23, x24, [sp, #16 * 2]
    ldp     x25, x26, [sp, #16 * 3]
    ldp     x27, x28, [sp, #16 * 4]
    ldp     x29, x30, [sp, #16 * 5]
    
    // Restore SP_EL0
    ldr     x10, [sp, #16 * 6]
    msr     sp_el0, x10

    add     sp, sp, #112

    // 5. Return (to the address in x30/LR of the new task)
    ret

// =============================================================================
// Enter User Mode
// =============================================================================
// void enter_user_mode(u64 entry, u64 stack);
// x0 = Entry Point
// x1 = Stack Pointer (SP_EL0)
.global enter_user_mode
enter_user_mode:
    // Mask all exceptions (D, A, I, F)
    msr     daifset, #0xf

    // SPSR_EL1 (Saved Program Status Register)
    // M[3:0] = 0000 -> EL0t
    msr     spsr_el1, xzr

    // ELR_EL1 (Exception Link Register) -> Entry Point
    msr     elr_el1, x0

    // SP_EL0 (Stack Pointer for EL0) -> Stack
    msr     sp_el0, x1

    // Zero out general purpose registers to prevent info leak (optional but good practice)
    mov     x0, xzr
    mov     x1, xzr
    mov     x2, xzr
    mov     x3, xzr
    mov     x4, xzr
    mov     x5, xzr
    mov     x6, xzr
    mov     x7, xzr
    mov     x8, xzr
    mov     x9, xzr
    mov     x10, xzr
    mov     x11, xzr
    mov     x12, xzr
    mov     x13, xzr
    mov     x14, xzr
    mov     x15, xzr
    mov     x16, xzr
    mov     x17, xzr
    mov     x18, xzr
    mov     x19, xzr
    mov     x20, xzr
    mov     x21, xzr
    mov     x22, xzr
    mov     x23, xzr
    mov     x24, xzr
    mov     x25, xzr
    mov     x26, xzr
    mov     x27, xzr
    mov     x28, xzr
    mov     x29, xzr
    mov     x30, xzr

    // Return from exception -> Jumps to EL0
    eret
