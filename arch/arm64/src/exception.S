// =============================================================================
// APRK OS - Exception Vector Table
// =============================================================================
// The exception vector table is an array of 16 entries.
// Each entry is 128 bytes (32 instructions) long.
// The base address must be 2048-byte aligned.
// =============================================================================

.section .text
.global exception_vector_table

.align 11 // 2^11 = 2048 bytes alignment required for VBAR_EL1

exception_vector_table:
    // -------------------------------------------------------------------------
    // Current EL with SP0 (We don't use this)
    // -------------------------------------------------------------------------
    .align 7 // 128 bytes
    b       unhandled_exception     // Synchronous
    .align 7
    b       unhandled_exception     // IRQ
    .align 7
    b       unhandled_exception     // FIQ
    .align 7
    b       unhandled_exception     // SError

    // -------------------------------------------------------------------------
    // Current EL with SPx (This is where the Kernel lives)
    // -------------------------------------------------------------------------
    .align 7
    b       sync_handler_entry      // Synchronous (e.g., SVC, Data Abort)
    .align 7
    b       irq_handler_entry       // IRQ (Interrupts like Timer, UART)
    .align 7
    b       unhandled_exception     // FIQ
    .align 7
    b       unhandled_exception     // SError

    // -------------------------------------------------------------------------
    // Lower EL using AArch64 (User Mode 64-bit)
    // -------------------------------------------------------------------------
    .align 7
    b       sync_handler_entry      // Synchronous (SVC) from Lower EL (EL0)
    .align 7
    b       irq_handler_entry       // IRQ (Timer) from Lower EL (EL0)
    .align 7
    b       unhandled_exception
    .align 7
    b       unhandled_exception

    // -------------------------------------------------------------------------
    // Lower EL using AArch32 (User Mode 32-bit - Not supported)
    // -------------------------------------------------------------------------
    .align 7
    b       unhandled_exception
    .align 7
    b       unhandled_exception
    .align 7
    b       unhandled_exception
    .align 7
    b       unhandled_exception

// =============================================================================
// Exception Entry/Exit Macros
// =============================================================================

// Save general purpose registers (x0-x30)
// Save general purpose registers (x0-x30)
// Save general purpose registers (x0-x30)
.macro SAVE_CONTEXT
    sub     sp, sp, #784
    stp     x0, x1, [sp, #16 * 0]
    stp     x2, x3, [sp, #16 * 1]
    stp     x4, x5, [sp, #16 * 2]
    stp     x6, x7, [sp, #16 * 3]
    stp     x8, x9, [sp, #16 * 4]
    stp     x10, x11, [sp, #16 * 5]
    stp     x12, x13, [sp, #16 * 6]
    stp     x14, x15, [sp, #16 * 7]
    stp     x16, x17, [sp, #16 * 8]
    stp     x18, x19, [sp, #16 * 9]
    stp     x20, x21, [sp, #16 * 10]
    stp     x22, x23, [sp, #16 * 11]
    stp     x24, x25, [sp, #16 * 12]
    stp     x26, x27, [sp, #16 * 13]
    stp     x28, x29, [sp, #16 * 14]
    
    // Save x30 (LR) and ELR_EL1
    mrs     x9, elr_el1
    stp     x30, x9, [sp, #16 * 15]

    // Save SPSR_EL1 and Padding (using xzr)
    mrs     x10, spsr_el1
    stp     x10, xzr, [sp, #16 * 16]

    // Save SIMD/FP registers (q0-q31) at offset 272 (16*17)
    stp     q0, q1, [sp, #272 + 32 * 0]
    stp     q2, q3, [sp, #272 + 32 * 1]
    stp     q4, q5, [sp, #272 + 32 * 2]
    stp     q6, q7, [sp, #272 + 32 * 3]
    stp     q8, q9, [sp, #272 + 32 * 4]
    stp     q10, q11, [sp, #272 + 32 * 5]
    stp     q12, q13, [sp, #272 + 32 * 6]
    stp     q14, q15, [sp, #272 + 32 * 7]
    stp     q16, q17, [sp, #272 + 32 * 8]
    stp     q18, q19, [sp, #272 + 32 * 9]
    stp     q20, q21, [sp, #272 + 32 * 10]
    stp     q22, q23, [sp, #272 + 32 * 11]
    stp     q24, q25, [sp, #272 + 32 * 12]
    stp     q26, q27, [sp, #272 + 32 * 13]
    stp     q28, q29, [sp, #272 + 32 * 14]
    stp     q30, q31, [sp, #272 + 32 * 15]
.endm

// Restore general purpose registers
.macro RESTORE_CONTEXT
    // Restore SIMD/FP registers (q0-q31)
    ldp     q0, q1, [sp, #272 + 32 * 0]
    ldp     q2, q3, [sp, #272 + 32 * 1]
    ldp     q4, q5, [sp, #272 + 32 * 2]
    ldp     q6, q7, [sp, #272 + 32 * 3]
    ldp     q8, q9, [sp, #272 + 32 * 4]
    ldp     q10, q11, [sp, #272 + 32 * 5]
    ldp     q12, q13, [sp, #272 + 32 * 6]
    ldp     q14, q15, [sp, #272 + 32 * 7]
    ldp     q16, q17, [sp, #272 + 32 * 8]
    ldp     q18, q19, [sp, #272 + 32 * 9]
    ldp     q20, q21, [sp, #272 + 32 * 10]
    ldp     q22, q23, [sp, #272 + 32 * 11]
    ldp     q24, q25, [sp, #272 + 32 * 12]
    ldp     q26, q27, [sp, #272 + 32 * 13]
    ldp     q28, q29, [sp, #272 + 32 * 14]
    ldp     q30, q31, [sp, #272 + 32 * 15]

    // Restore SPSR_EL1
    ldr     x10, [sp, #16 * 16]
    msr     spsr_el1, x10

    // Restore x30 and ELR_EL1
    ldp     x30, x9, [sp, #16 * 15]
    msr     elr_el1, x9

    // Restore GP registers
    ldp     x0, x1, [sp, #16 * 0]
    ldp     x2, x3, [sp, #16 * 1]
    ldp     x4, x5, [sp, #16 * 2]
    ldp     x6, x7, [sp, #16 * 3]
    ldp     x8, x9, [sp, #16 * 4]
    ldp     x10, x11, [sp, #16 * 5]
    ldp     x12, x13, [sp, #16 * 6]
    ldp     x14, x15, [sp, #16 * 7]
    ldp     x16, x17, [sp, #16 * 8]
    ldp     x18, x19, [sp, #16 * 9]
    ldp     x20, x21, [sp, #16 * 10]
    ldp     x22, x23, [sp, #16 * 11]
    ldp     x24, x25, [sp, #16 * 12]
    ldp     x26, x27, [sp, #16 * 13]
    ldp     x28, x29, [sp, #16 * 14]
    
    add     sp, sp, #784
.endm

// =============================================================================
// Handler Wrappers
// =============================================================================

sync_handler_entry:
    SAVE_CONTEXT
    mov     x0, sp              // Pass trap frame pointer as arg0
    bl      handle_sync_exception
    RESTORE_CONTEXT
    eret

irq_handler_entry:
    SAVE_CONTEXT
    bl      handle_irq_exception
    RESTORE_CONTEXT
    eret

unhandled_exception:
    // Infinite loop for now
    wfe
    b       unhandled_exception
