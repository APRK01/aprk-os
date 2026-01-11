// =============================================================================
// APRK OS - ARM64 Boot Assembly
// =============================================================================
// This is the first code that runs when APRK OS boots on ARM64.
// It sets up the initial environment and jumps to Rust code.
//
// Entry point: _start
// Target: QEMU virt machine (ARM64)
// =============================================================================

.section .text._start
.global _start

_start:
    // -------------------------------------------------------------------------
    // Step 1: Check processor ID - only boot on CPU 0
    // -------------------------------------------------------------------------
    // On multi-core systems, all cores start executing here.
    // We only want core 0 to continue; others should halt.
    mrs     x0, mpidr_el1           // Read Multiprocessor Affinity Register
    and     x0, x0, #0xFF           // Extract CPU ID (Aff0 field)
    cbnz    x0, halt                // If not CPU 0, halt

    // -------------------------------------------------------------------------
    // Step 1.5: Switch from EL2 to EL1 (if necessary)
    // -------------------------------------------------------------------------
    // QEMU starts in EL2 by default. We want to run the kernel in EL1.
    mrs     x0, CurrentEL
    and     x0, x0, #0b1100         // Get EL bits
    cmp     x0, #0b1000             // Check if EL2 (0x8)
    b.ne    el1_entry               // If not EL2, assume EL1 and continue

    // We are in EL2. Configure EL1 for AArch64 and drop.
    
    // HCR_EL2: RW (Bit 31) = 1 (EL1 is AArch64)
    mov     x0, #(1 << 31)
    msr     hcr_el2, x0
    
    // SPSR_EL2: Mask Attributes (D,A,I,F) | Mode EL1h (0b0101)
    // 0x3C5 = 0011 1100 0101
    // D=1, A=1, I=1, F=1, M=0101 (EL1h)
    mov     x0, #0x3c5
    msr     spsr_el2, x0
    
    // ELR_EL2: Return address is 'el1_entry'
    adr     x0, el1_entry
    msr     elr_el2, x0
    
    // Return to EL1
    eret

el1_entry:

    // -------------------------------------------------------------------------
    // Step 2: Set up the stack pointer
    // -------------------------------------------------------------------------
    // The stack grows downward, so we point to the top of our stack area.
    // Stack is defined in the linker script.
    adrp    x0, __stack_top         // Load page address of stack top
    add     x0, x0, :lo12:__stack_top
    mov     sp, x0                  // Set stack pointer

    // -------------------------------------------------------------------------
    // Step 3: Clear the BSS section
    // -------------------------------------------------------------------------
    // BSS contains uninitialized global variables; we must zero them.
    adrp    x0, __bss_start         // Start of BSS
    add     x0, x0, :lo12:__bss_start
    adrp    x1, __bss_end           // End of BSS
    add     x1, x1, :lo12:__bss_end

bss_clear_loop:
    cmp     x0, x1                  // Check if we've reached the end
    b.ge    bss_clear_done          // If so, we're done
    str     xzr, [x0], #8           // Store zero and increment pointer
    b       bss_clear_loop          // Repeat

bss_clear_done:
    // -------------------------------------------------------------------------
    // Step 3.5: Enable FPU/SIMD (Required for Rust and Context Switch)
    // -------------------------------------------------------------------------
    // CPACR_EL1 bits [21:20] must be set to 0b11 to enable FP/SIMD at EL0 and EL1.
    // Otherwise, ANY access to q0-q31 (context save or memcpy) traps!
    mov     x0, #(3 << 20)
    msr     cpacr_el1, x0
    isb                             // Instruction Synchronization Barrier

    // -------------------------------------------------------------------------
    // Step 4: Jump to Rust kernel entry point
    // -------------------------------------------------------------------------
    // At this point:
    // - We're running on CPU 0 only
    // - Stack is set up
    // - BSS is zeroed
    // Time to hand control to Rust!
    bl      kernel_main             // Call the Rust entry point

    // -------------------------------------------------------------------------
    // If kernel_main returns (it shouldn't), halt the CPU
    // -------------------------------------------------------------------------
halt:
    wfe                             // Wait for event (low power halt)
    b       halt                    // Loop forever

// =============================================================================
// End of boot.S
// =============================================================================
