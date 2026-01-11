// =============================================================================
// APRK OS - Interactive Shell (Premium)
// =============================================================================

use aprk_arch_arm64::{print, println, uart};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub extern "C" fn run() {
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
        // Use if-let to poll.
        if let Some(c) = uart::get_char() {
            match c {
                b'\n' | b'\r' => {
                    println!();
                    let cmd = buffer.trim().to_string();
                    if !cmd.is_empty() {
                         // Add to history
                         if history.len() >= 10 { history.remove(0); }
                         history.push(cmd.clone());
                         execute_command(&cmd);
                    }
                    buffer.clear();
                    print_prompt();
                }
                8 | 127 => {
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
             // Yield to scheduler if no input
             crate::sched::schedule();
             // Hint to CPU we are in a busy-wait loop (optional, but good)
             core::hint::spin_loop();
        }
    }
}

fn print_prompt() {
    print!("\x1b[1;32mroot@aprk\x1b[0m:\x1b[1;34m/\x1b[0m$ ");
}

fn execute_command(cmd_line: &str) {
    let mut parts = cmd_line.split_whitespace();
    let cmd = match parts.next() {
        Some(c) => c,
        None => return,
    };
    let args: Vec<&str> = parts.collect();

    match cmd {
        "help" => show_help(),
        "fetch" | "neofetch" => show_neofetch(),
        "clear" => print!("\x1b[2J\x1b[1;1H"),
        "whoami" => println!("root"),
        "uname" => println!("APRK OS 1.0.0 Genesis aarch64"),
        "ls" => {
            println!();
            crate::fs::ls(crate::fs::RAMDISK);
            println!();
        },
        "exec" => {
             if let Some(filename) = args.get(0) {
                 if let Some(file) = crate::fs::get_file(crate::fs::RAMDISK, filename) {
                     println!("Loading '{}'...", filename);
                     unsafe {
                         if let Some(entry) = crate::loader::load_elf(file.data) {
                             // entry is u64 address.
                             crate::sched::spawn_user(entry, filename);
                         }
                     }
                 } else {
                     println!("Not found");
                 }
             }
        },

        _ => println!("Unknown command"),
    }
}

fn show_neofetch() {
    println!();
    print!("\x1b[1;36m");
    println!(r"   __  __ ");
    println!(r"   / \   OS:      APRK OS (Genesis)");
    println!(r"  / _ \  Kernel:  v1.0.0");
    println!(r" / ___ \ Arch:    ARM64 (AArch64)");
    println!(r"/_/   \_\Shell:   APRK Premium Shell");
    print!("\x1b[0m");
    println!();
}

fn show_help() {
    println!("\nAvailable commands: fetch, ls, exec, help, clear");
}
