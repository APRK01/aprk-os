// =============================================================================
// APRK OS - Interactive Shell (Premium)
// =============================================================================

use aprk_arch_arm64::{print, println, uart};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::sched;

pub extern "C" fn shell_task() {
    unsafe { aprk_arch_arm64::cpu::enable_interrupts(); }

    print!("\x1b[2J\x1b[1;1H");
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    APRK OS Shell v1.0                         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Welcome! Type 'help' for available commands.");
    println!();
    print_prompt();

    let mut buffer = String::new();
    let mut history: Vec<String> = Vec::new();

    loop {
        if let Some(c) = uart::get_char() {
            match c {
                b'\n' | b'\r' => {
                    println!();
                    let cmd_line = buffer.trim().to_string();
                    if !cmd_line.is_empty() {
                         if history.len() >= 10 { history.remove(0); }
                         history.push(cmd_line.clone());
                         execute_command(&cmd_line);
                    }
                    buffer.clear();
                    print_prompt();
                }
                b'\x08' | 127 => { // Backspace
                    if !buffer.is_empty() {
                         buffer.pop();
                         print!("\x08 \x08");
                    }
                }
                _ => {
                    buffer.push(c as char);
                    print!("{}", c as char);
                }
            }
        } else {
             sched::schedule();
             core::hint::spin_loop();
        }
    }
}

fn print_prompt() {
    print!("\x1b[1;32mroot@aprk\x1b[0m:\x1b[1;34m/\x1b[0m$ ");
}

fn execute_command(cmd_line: &str) {
    let parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() { return; }
    
    match parts[0] {
        "help" => {
            println!("Available commands:");
            println!("  help      - Show this help message");
            println!("  version   - Show OS version info");
            println!("  ls        - List files on disk");
            println!("  cat <f>   - Print file content");
            println!("  exec <f>  - Execute an ELF binary");
            println!("  ps        - List running tasks");
            println!("  clear     - Clear the screen");
        },
        "version" => {
            println!("APRK OS v1.0 (FAT32 Enabled)");
        },
        "ls" => {
            crate::fs::list_root();
        },
        "ps" => {
            sched::print_tasks();
        },
        "cat" => {
            if parts.len() < 2 {
                println!("Usage: cat <filename>");
            } else {
                let filename = parts[1];
                if let Some(content) = crate::fs::read_file(filename) {
                    if let Ok(s) = core::str::from_utf8(&content) {
                        println!("{}", s);
                    } else {
                        println!("[shell] Error: File is binary or invalid UTF-8");
                    }
                } else {
                    println!("[shell] Error: File not found");
                }
            }
        },
        "exec" => {
            if parts.len() < 2 {
                println!("Usage: exec <binary_name>");
            } else {
                let binary_name = parts[1];
                println!("[shell] Executing {}...", binary_name);
                
                if let Some(elf_data) = crate::fs::read_file(binary_name) {
                    unsafe {
                        if let Some(entry_point) = crate::loader::load_elf(&elf_data) {
                            println!("[shell] Starting process at {:#x}", entry_point);
                            sched::spawn_user(entry_point, binary_name);
                        } else {
                            println!("[shell] Error: Failed to load ELF");
                        }
                    }
                } else {
                    println!("[shell] Error: Binary not found");
                }
            }
        },
        "clear" => {
            print!("\x1b[2J\x1b[H"); 
        },
        _ => {
            println!("Unknown command: {}", parts[0]);
        }
    }
}
