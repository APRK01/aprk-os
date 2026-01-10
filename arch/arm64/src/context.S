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
    // 1. Save Callee-Saved Registers (x19-x30)
    sub     sp, sp, #96
    stp     x19, x20, [sp, #16 * 0]
    stp     x21, x22, [sp, #16 * 1]
    stp     x23, x24, [sp, #16 * 2]
    stp     x25, x26, [sp, #16 * 3]
    stp     x27, x28, [sp, #16 * 4]
    stp     x29, x30, [sp, #16 * 5]  // x29=FP, x30=LR

    // 2. Save current SP to *x0
    mov     x19, sp
    str     x19, [x0]

    // 3. Load next SP from x1
    mov     sp, x1

    // 4. Restore Callee-Saved Registers
    ldp     x19, x20, [sp, #16 * 0]
    ldp     x21, x22, [sp, #16 * 1]
    ldp     x23, x24, [sp, #16 * 2]
    ldp     x25, x26, [sp, #16 * 3]
    ldp     x27, x28, [sp, #16 * 4]
    ldp     x29, x30, [sp, #16 * 5]
    add     sp, sp, #96

    // 5. Return (to the address in x30/LR of the new task)
    ret
