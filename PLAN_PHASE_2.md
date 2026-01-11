# APRK OS Phase 2: Core, Interaction, & Storage

This document outlines the roadmap for the next major evolution of APRK OS.

## 🟢 Phase 1: Core Hardening (The Stabilizer)
**Goal:** Run the scheduler at 50ms (20Hz) without crashing, enabling smooth multitasking.

*   [ ] **Debug Save/Restore**: The current crash (`ELR=0x4`) suggests register corruption during context switches.
    *   *Hypothesis*: SIMD/FP registers are not being saved, but Rust `String`/`memcpy` uses them.
*   [ ] **Implement SIMD Context Switching**: Modify `exception.S` and `context_switch` to save `q0-q31`.
*   [ ] **Stress Test**: Revert timer to 50ms and run the Premium Shell.

## 🟡 Phase 2: Interactive Userspace (The Playground)
**Goal:** Run real, interactive applications (games) in userspace.

*   [ ] **Heap Syscall (`sys_brk`)**: Implement a syscall to allow user programs to request memory (enabling `Vec` and `Box` in userspace).
*   [ ] **Input Syscall (`sys_read`)**: Allow `hello` program to read keystrokes from the kernel.
*   [ ] **Game Demo**: Port a simple game (e.g., **Snake**) to APRK OS userspace to prove interactivity.

## 🔵 Phase 3: True File System (The Workbench)
**Goal:** Create, modify, and delete files.

*   [ ] **Writable VFS**: Upgrade from Read-Only TarFS to a Mutable RAMFS.
*   [ ] **File Operations**: Implement `sys_open`, `sys_write`, `sys_close`.
*   [ ] **Shell Commands**: Add `touch`, `cp`, `rm`.
*   [ ] **Text Editor**: Create a minimal editor (like `nano`) to write code inside the OS.
